/**
 * Feedback Data & Helpers
 * Shared between SentenceBuilder and CommentInput.
 */

import { getAblativeSuffix } from './turkish.js';

/** Replace template tokens in a label string. */
export function resolveLabel(template, foodName) {
  return template.replace('{ABL}', getAblativeSuffix(foodName));
}

export const sentiments = [
  { id: 1, label: " yenur", sentiment: "positive" },
  { id: 2, label: " şaşırttı", sentiment: "positive" },
  { id: 3, label: " tam bir şifa", sentiment: "positive" },
  { id: 4, label: " protein bombası", sentiment: "positive" },
  { id: 5, label: " süper olmuş", sentiment: "positive" },
  { id: 6, label: " efsane", sentiment: "positive" },
  { id: 7, label: " doyurucu", sentiment: "positive" },
  { id: 8, label: " harika görünüyor", sentiment: "positive" },
  { id: 9, label: " favorim", sentiment: "positive" },
  { id: 10, label: " bugün çok güzel", sentiment: "positive" },
  { id: 11, label: "{ABL} bıktım artık", sentiment: "negative" },
  { id: 12, label: " rezaletti", sentiment: "negative" },
  { id: 13, label: " mide fesadı", sentiment: "negative" },
  { id: 14, label: " yenmez", sentiment: "negative" },
  { id: 15, label: " tatsızdı", sentiment: "negative" },
  { id: 16, label: " berbattı", sentiment: "negative" },
  { id: 17, label: " kötüydü", sentiment: "negative" },
  //  { id: 18, label: " yine mi?!", sentiment: "negative" },
  { id: 19, label: " olmamış", sentiment: "negative" },
  { id: 20, label: " yiyeceklere yazık olmuş", sentiment: "negative" }
];

export const recommendations = [
  { id: 21, label: "Olay yerinde olacağım", sentiment: "positive" },
  //  { id: 22, label: "Algötür vakti", sentiment: "neutral" },
  { id: 23, label: "Dışarıdan söyleyin", sentiment: "negative" },
  { id: 24, label: "Uzak durun", sentiment: "negative" },
  { id: 25, label: "Ekmek arası yapın", sentiment: "neutral" },
  { id: 26, label: "Koşun gelin", sentiment: "positive" },
  { id: 27, label: "İdare eder", sentiment: "neutral" },
  { id: 28, label: "Boykot zamanı", sentiment: "negative" },
  //  { id: 29, label: "Sahura saklayın", sentiment: "neutral" },
  { id: 30, label: "Herkesi davet ediyorum", sentiment: "positive" }
];
