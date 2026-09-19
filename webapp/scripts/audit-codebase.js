import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { parse, parseCss } from 'svelte/compiler';

const SRC_DIR = join(import.meta.dirname, '..', 'src');
const STYLES_DIR = join(SRC_DIR, 'styles');

const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const BOLD = '\x1b[1m';
const RESET = '\x1b[0m';

async function findFiles(dir, extensions) {
  const files = [];
  try {
    const entries = await readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = join(dir, entry.name);
      if (entry.isDirectory()) {
        files.push(...(await findFiles(fullPath, extensions)));
      } else if (extensions.some(ext => entry.name.endsWith(ext))) {
        files.push(fullPath);
      }
    }
  } catch {
    // Directory might not exist
  }
  return files;
}

function walkAst(node, visitor, parent = null) {
  if (!node || typeof node !== 'object') return;
  visitor(node, parent);
  for (const key of Object.keys(node)) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child) walkAst(item, visitor, node);
    } else if (child && typeof child === 'object') {
      walkAst(child, visitor, node);
    }
  }
}

async function extractClassesFromCss(filePath) {
  const content = await readFile(filePath, 'utf-8');
  const classMap = new Map(); // className -> { file, line }
  try {
    const ast = parseCss(content, 0);
    walkAst(ast, (node) => {
      if (node.type === 'ClassSelector' && node.name) {
        if (!classMap.has(node.name)) {
          const line = content.slice(0, node.start).split('\n').length;
          classMap.set(node.name, { file: filePath, line });
        }
      }
    });
  } catch {
    // Non-standard CSS or syntax fallback
  }
  return classMap;
}

function checkInlineStyles(ast, relPath) {
  const violations = [];

  walkAst(ast.html, (node) => {
    if (node.type === 'Attribute' && node.name === 'style') {
      const line = node.name_loc?.start?.line ?? 1;
      let styleText = '';

      if (Array.isArray(node.value)) {
        for (const part of node.value) {
          if (part.type === 'Text') {
            styleText += part.data;
          } else if (part.type === 'ExpressionTag') {
            styleText += ' {dynamic} ';
          }
        }
      }

      const decls = styleText.split(';').map(d => d.trim()).filter(Boolean);
      for (const decl of decls) {
        const colonIdx = decl.indexOf(':');
        if (colonIdx === -1) continue;
        const prop = decl.slice(0, colonIdx).trim();

        if (!prop.startsWith('--')) {
          violations.push({
            file: relPath,
            line,
            decl: decl.slice(0, 50),
            reason: `Statik CSS kuralı ("${prop}") tespit edildi. style="..." içinde sadece "--*" custom property kullanılabilir.`
          });
        }
      }
    }
  });

  return violations;
}

function checkMemoryHygiene(ast, relPath) {
  const issues = [];
  let hasSvelteWindow = false;
  let hasSvelteDocument = false;

  // Check if declarative <svelte:window> or <svelte:document> tags exist in template
  walkAst(ast.html, (node) => {
    if (node.type === 'Window') hasSvelteWindow = true;
    if (node.type === 'Document') hasSvelteDocument = true;
  });

  const checkScript = (scriptNode) => {
    if (!scriptNode) return;

    const timers = [];
    const subscriptions = [];
    const clearedTimerHandles = new Set();
    const calledCleanupHandles = new Set();

    // 1. Collect cleanup scopes (onDestroy callbacks, onMount/effect return functions)
    const inspectCleanupScope = (fnNode) => {
      if (!fnNode) return;
      walkAst(fnNode, (node) => {
        if (node.type === 'CallExpression') {
          // clearInterval(handle)
          if (node.callee?.name === 'clearInterval') {
            const arg = node.arguments?.[0];
            if (arg?.type === 'Identifier') {
              clearedTimerHandles.add(arg.name);
            }
          }
          // unsub()
          if (node.callee?.type === 'Identifier') {
            calledCleanupHandles.add(node.callee.name);
          }
        }
      });
    };

    walkAst(scriptNode, (node, parent) => {
      if (node.type !== 'CallExpression') return;

      const callee = node.callee;
      const line = node.loc?.start?.line ?? 1;

      // Track onDestroy(fn)
      if (callee?.name === 'onDestroy') {
        inspectCleanupScope(node.arguments?.[0]);
      }

      // Track onMount / $effect return cleanup: return () => { ... }
      if (callee?.name === 'onMount' || callee?.name === '$effect') {
        const fn = node.arguments?.[0];
        if (fn) {
          walkAst(fn, (child) => {
            if (child.type === 'ReturnStatement' && child.argument) {
              inspectCleanupScope(child.argument);
            }
          });
        }
      }

      // Track setInterval
      if (callee?.name === 'setInterval') {
        let handle = null;
        if (parent?.type === 'AssignmentExpression' && parent.left?.type === 'Identifier') {
          handle = parent.left.name;
        } else if (parent?.type === 'VariableDeclarator' && parent.id?.type === 'Identifier') {
          handle = parent.id.name;
        }
        timers.push({ handle, line });
      }

      // Track store.subscribe(...)
      if (callee?.type === 'MemberExpression' && callee.property?.name === 'subscribe') {
        let handle = null;
        if (parent?.type === 'AssignmentExpression' && parent.left?.type === 'Identifier') {
          handle = parent.left.name;
        } else if (parent?.type === 'VariableDeclarator' && parent.id?.type === 'Identifier') {
          handle = parent.id.name;
        }
        subscriptions.push({ handle, line });
      }

      // Track naked window.addEventListener(...)
      if (
        callee?.type === 'MemberExpression' &&
        callee.object?.name === 'window' &&
        callee.property?.name === 'addEventListener'
      ) {
        const firstArg = node.arguments?.[0];
        const eventName = firstArg?.value;
        const standardDomEvents = ['resize', 'scroll', 'popstate', 'keydown', 'keyup', 'click', 'offline', 'online', 'mousemove', 'mouseup', 'touchmove', 'touchend'];

        if (typeof eventName === 'string' && standardDomEvents.includes(eventName) && !hasSvelteWindow) {
          issues.push({
            file: relPath,
            line,
            detail: `Çıplak window.addEventListener("${eventName}") yerine deklaratif <svelte:window on${eventName}={...} /> kullanın. Svelte bileşeni sökülürken otomatik temizler.`
          });
        }
      }
    });

    // 2. Semantic verification of timers
    for (const t of timers) {
      if (!t.handle) {
        issues.push({
          file: relPath,
          line: t.line,
          detail: `setInterval çağrısı herhangi bir değişkene atanmamış; handle saklanmadığı için yaşam döngüsünde clearInterval ile temizlenmesi imkansız.`
        });
      } else if (!clearedTimerHandles.has(t.handle)) {
        issues.push({
          file: relPath,
          line: t.line,
          detail: `"${t.handle}" zamanlayıcısı başlatılmış ancak onDestroy veya $effect/onMount cleanup fonksiyonu içinde clearInterval(${t.handle}) ile temizlenmemiş.`
        });
      }
    }

    // 3. Semantic verification of subscriptions
    for (const s of subscriptions) {
      if (!s.handle) {
        issues.push({
          file: relPath,
          line: s.line,
          detail: `store.subscribe() çağrısı unsubscribe değişkenine atanmamış; yaşam döngüsünde sonlandırılmadığı için bellek sızıntısı riski taşır.`
        });
      } else if (!calledCleanupHandles.has(s.handle)) {
        issues.push({
          file: relPath,
          line: s.line,
          detail: `"${s.handle}" aboneliği saklanmış ancak onDestroy veya cleanup fonksiyonu içinde ${s.handle}() ile çağrılmamış.`
        });
      }
    }
  };

  checkScript(ast.instance);
  checkScript(ast.module);

  return issues;
}

function extractTemplateClasses(ast) {
  const classes = new Set();

  walkAst(ast.html, (node) => {
    if (node.type === 'Attribute' && node.name === 'class') {
      if (Array.isArray(node.value)) {
        for (const part of node.value) {
          if (part.type === 'Text') {
            for (const c of part.data.trim().split(/\s+/)) {
              // Ignore incomplete interpolation prefixes ending with hyphen (e.g. "btn--", "badge--")
              if (c && !c.endsWith('-') && /^[a-zA-Z][a-zA-Z0-9_-]*$/.test(c)) {
                classes.add(c);
              }
            }
          }
        }
      }
    } else if (node.type === 'Class') {
      classes.add(node.name);
    }
  });

  return classes;
}

async function main() {
  console.log(`${BOLD}${CYAN}=== Kepçe AST Tabanlı Kod ve Stil Denetimi (Svelte/Compiler) ===${RESET}\n`);

  let errorCount = 0;
  let warningCount = 0;

  const svelteFiles = await findFiles(SRC_DIR, ['.svelte']);
  const jsFiles = await findFiles(SRC_DIR, ['.js', '.ts']);

  const inlineStyleViolations = [];
  const memoryIssues = [];
  const usedClasses = new Set();
  const rawContents = [];
  const scopedClasses = new Set();

  for (const file of jsFiles) {
    const code = await readFile(file, 'utf-8');
    rawContents.push(code);
    // Only match classList operations or className assignments
    const classListMatches = code.matchAll(/classList\.(?:add|remove|toggle|contains)\(\s*['"`]([a-zA-Z][a-zA-Z0-9_-]*)['"`]/g);
    for (const m of classListMatches) {
      usedClasses.add(m[1]);
    }
    const classNameMatches = code.matchAll(/\bclassName\s*=\s*['"`]([a-zA-Z0-9_ -]+)['"`]/g);
    for (const m of classNameMatches) {
      for (const c of m[1].trim().split(/\s+/)) {
        if (/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(c) && !c.endsWith('-')) usedClasses.add(c);
      }
    }
  }

  for (const file of svelteFiles) {
    const relPath = relative(SRC_DIR, file);
    const code = await readFile(file, 'utf-8');
    rawContents.push(code);

    try {
      const ast = parse(code);
      inlineStyleViolations.push(...checkInlineStyles(ast, relPath));
      memoryIssues.push(...checkMemoryHygiene(ast, relPath));

      for (const cls of extractTemplateClasses(ast)) {
        usedClasses.add(cls);
      }

      if (ast.css) {
        walkAst(ast.css, (node) => {
          if (node.type === 'ClassSelector' && node.name) {
            scopedClasses.add(node.name);
          }
        });
      }
    } catch (err) {
      console.log(`  ${RED}✗ AST Parse Hatası:${RESET} [${relPath}] ${err.message}`);
      errorCount++;
    }
  }

  // 1. Inline Style Check
  console.log(`${BOLD}1. Statik Inline Stil Denetimi (AST)${RESET}`);
  if (inlineStyleViolations.length === 0) {
    console.log(`  ${GREEN}✓ Temiz:${RESET} Tüm style="..." attribute'ları AST düzeyinde incelendi; kurala uygun (--custom-prop pattern).`);
  } else {
    for (const v of inlineStyleViolations) {
      console.log(`  ${RED}✗ HATA:${RESET} [${v.file}:${v.line}] ${v.reason} ("${v.decl}")`);
      errorCount++;
    }
  }
  console.log('');

  // 2. Memory Leak Check (Semantic Scope Tracing)
  console.log(`${BOLD}2. Bellek Sızıntısı & Yaşam Döngüsü Denetimi (AST Scope Tracing)${RESET}`);
  if (memoryIssues.length === 0) {
    console.log(`  ${GREEN}✓ Temiz:${RESET} Çıplak DOM event listener veya sızıntılı zamanlayıcı riski bulunamadı.`);
  } else {
    for (const m of memoryIssues) {
      console.log(`  ${YELLOW}⚠ UYARI:${RESET} [${m.file}:${m.line}] ${m.detail}`);
      warningCount++;
    }
  }
  console.log('');

  // 3. CSS Class Parity Check
  console.log(`${BOLD}3. Stil Paritesi & Ölü/Hayalet Sınıf Denetimi (parseCss AST)${RESET}`);

  // Collect classes defined in new styles/
  const newCssFiles = await findFiles(STYLES_DIR, ['.css']);
  const definedNewClasses = new Map(); // className -> { file, line }
  for (const file of newCssFiles) {
    const classMap = await extractClassesFromCss(file);
    for (const [cls, loc] of classMap.entries()) {
      if (!definedNewClasses.has(cls)) {
        definedNewClasses.set(cls, loc);
      }
    }
  }

  const combinedCode = rawContents.join(' ');

  // Dynamic class prefixes generated via template interpolation or utilities
  const DYNAMIC_PREFIXES = [
    'badge--', 'profile-flair--', 'audit-system-badge--', 'incident-card--',
    'status-card--', 'pill--', 'contributor-card__rank--', 'rich-status-tooltip__badge--',
    'tag-stat-fill--', 'coverage-cell--', 'status-summary--', 'c-skeleton--', 'skeleton--',
    'c-spinner--', 'u-syntax-', 'dropdown--', 'dropdown__trigger--', 'meal-card__source--',
    'banner--', 'c-modal--', 'btn--', 'theme-', 'gen-', 'ambient-badge--', 'settings-card--',
    'audit-progress-fill--', 'dev-table-tier-', 'l-grid--', 'gen-stack--', 'gen-cluster--',
    'gen-grid--', 'gen-card--', 'gen-metric__delta--', 'gen-badge--', 'u-',
    'c-', 'ci-',
    'is-', 'depth-', 'card--', 'pagination__btn--', 'stats-section-title--',
    'archive-row__votes--', 'admin-table-cell--pill--', 'action-btn--',
    'comment-node__badge--'
  ];

  const DESIGN_TOKENS = new Set([
    'l-grid', 'color-info', 'text-xl', 'font-weight-black', 'tabular-nums', 'ambient-badge'
  ]);

  // Dead CSS: Defined in new styles/ but NEVER used anywhere in src/
  const deadClasses = [];
  const deadByFile = new Map();

  for (const [cls, loc] of definedNewClasses.entries()) {
    if (DYNAMIC_PREFIXES.some(p => cls.startsWith(p))) continue;
    if (DESIGN_TOKENS.has(cls)) continue;
    if (!usedClasses.has(cls) && !combinedCode.includes(cls)) {
      deadClasses.push(cls);
      const relFile = relative(STYLES_DIR, loc.file);
      if (!deadByFile.has(relFile)) deadByFile.set(relFile, []);
      deadByFile.get(relFile).push({ cls, line: loc.line });
    }
  }

  if (deadClasses.length === 0) {
    console.log(`  ${GREEN}✓ Temiz:${RESET} styles/ altındaki tüm CSS sınıfları şablonlarda aktif kullanılıyor.`);
  } else {
    console.log(`  ${YELLOW}⚠ ÖLÜ CSS:${RESET} ${deadClasses.length} adet kullanılmayan sınıf tespit edildi:\n`);
    for (const [file, items] of Array.from(deadByFile.entries()).sort((a, b) => b[1].length - a[1].length)) {
      console.log(`    ${BOLD}${file}${RESET} (${items.length} sınıf):`);
      for (const item of items) {
        console.log(`      L${item.line.toString().padEnd(4)} .${item.cls}`);
      }
      console.log('');
    }
    warningCount += deadClasses.length;
  }

  // Action hooks and state markers used in JS/DOM logic (not visual CSS classes)
  const ACTION_HOOK_REGEX = /^(btn-(manage|edit|delete|key)-|(toggle|ban|warn|edit|delete|add|select|remove)-)/;
  const STATE_MARKERS = new Set(['enhanced', 'is-logged-in']);

  // Ghost Classes: Used in Svelte templates but not defined in active styles/ or scoped Svelte <style>
  const ghostClasses = [];
  for (const cls of usedClasses) {
    if (cls.startsWith('s-') || cls.startsWith('svelte-') || cls === 'dark' || cls === 'light') continue;
    if (ACTION_HOOK_REGEX.test(cls) || STATE_MARKERS.has(cls)) continue;
    if (!definedNewClasses.has(cls) && !scopedClasses.has(cls)) {
      ghostClasses.push(cls);
    }
  }

  if (ghostClasses.length === 0) {
    console.log(`  ${GREEN}✓ Temiz:${RESET} AST'den çıkarılan ${usedClasses.size} sınıfın tamamının CSS karşılığı mevcut.`);
  } else {
    console.log(`  ${YELLOW}⚠ HAYALET SINIF:${RESET} ${ghostClasses.length} adet CSS karşılığı bulunamayan sınıf:`);
    console.log(`    ${ghostClasses.join(', ')}`);
    warningCount += ghostClasses.length;
  }

  console.log(`\n${BOLD}Özet:${RESET} ${errorCount} Hata, ${warningCount} Uyarı`);

  if (errorCount > 0) {
    console.log(`${RED}${BOLD}DENETİM BAŞARISIZ!${RESET}`);
    process.exit(1);
  } else {
    console.log(`${GREEN}${BOLD}DENETİM BAŞARILI.${RESET}`);
    process.exit(0);
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
