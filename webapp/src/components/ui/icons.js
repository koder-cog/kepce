import dice from '../../assets/icons/dice.svg?raw';
import voteUp from '../../assets/icons/vote-up.svg?raw';
import voteUpFilled from '../../assets/icons/vote-up-filled.svg?raw';
import voteDown from '../../assets/icons/vote-down.svg?raw';
import voteDownFilled from '../../assets/icons/vote-down-filled.svg?raw';
import chevronDown from '../../assets/icons/chevron-down.svg?raw';
import chevronUp from '../../assets/icons/chevron-up.svg?raw';
import chevronLeft from '../../assets/icons/chevron-left.svg?raw';
import chevronRight from '../../assets/icons/chevron-right.svg?raw';
import chat from '../../assets/icons/chat.svg?raw';
import warning from '../../assets/icons/warning.svg?raw';
import strongLanguage from '../../assets/icons/strong-language.svg?raw';
import info from '../../assets/icons/info.svg?raw';
import infoCritical from '../../assets/icons/info-critical.svg?raw';
import user from '../../assets/icons/user.svg?raw';
import settings from '../../assets/icons/settings.svg?raw';
import logout from '../../assets/icons/logout.svg?raw';
import login from '../../assets/icons/login.svg?raw';
import moon from '../../assets/icons/moon.svg?raw';
import sun from '../../assets/icons/sun.svg?raw';
import externalLink from '../../assets/icons/external-link.svg?raw';
import logo from '../../assets/icons/logo.svg?raw';
import logoSmall from '../../assets/icons/logo-small.svg?raw';
import logoExperimental from '../../assets/icons/logo-experimental.svg?raw';
import logoSmallExperimental from '../../assets/icons/logo-small-experimental.svg?raw';
import system from '../../assets/icons/system.svg?raw';
import verified from '../../assets/icons/verified.svg?raw';
import star from '../../assets/icons/star.svg?raw';
import starFilled from '../../assets/icons/star-filled.svg?raw';
import starFilledHalfLeft from '../../assets/icons/star-filled-half-left.svg?raw';
import starFilledHalfRight from '../../assets/icons/star-filled-half-right.svg?raw';
import menuMissing from '../../assets/icons/menu-missing.svg?raw';
import crossSmall from '../../assets/icons/cross-small.svg?raw';
import avatarEmpty from '../../assets/icons/avatar-empty.svg?raw';
import calendar from '../../assets/icons/calendar.svg?raw';
import noConnection from '../../assets/icons/no-connection.svg?raw';
import votedUpMore from '../../assets/icons/voted-upmore.svg?raw';
import votedDownMore from '../../assets/icons/voted-downmore.svg?raw';
import votedEqual from '../../assets/icons/voted-equal.svg?raw';
import votedNone from '../../assets/icons/voted-none.svg?raw';
import cards from '../../assets/icons/cards.svg?raw';
import check from '../../assets/icons/check.svg?raw';
import wheat from '../../assets/icons/wheat.svg?raw';
import utensils from '../../assets/icons/utensils.svg?raw';
import keyboard from '../../assets/icons/keyboard.svg?raw';
import ghost from '../../assets/icons/ghost.svg?raw';
import timeout from '../../assets/icons/timeout.svg?raw';
import creditCard from '../../assets/icons/credit-card.svg?raw';
import server from '../../assets/icons/server.svg?raw';
import bread from '../../assets/icons/bread.svg?raw';
import combine from '../../assets/icons/combine.svg?raw';
import split from '../../assets/icons/split.svg?raw';
import edit from '../../assets/icons/edit.svg?raw';
import soup from '../../assets/icons/soup.svg?raw';
import meat from '../../assets/icons/meat.svg?raw';
import fish from '../../assets/icons/fish.svg?raw';
import egg from '../../assets/icons/egg.svg?raw';
import dessert from '../../assets/icons/dessert.svg?raw';
import tea from '../../assets/icons/tea.svg?raw';
import garlic from '../../assets/icons/garlic.svg?raw';
import pepper from '../../assets/icons/pepper.svg?raw';
import nut from '../../assets/icons/nut.svg?raw';
import salami from '../../assets/icons/salami.svg?raw';
import commentNone from '../../assets/icons/comment-none.svg?raw';
import eyeLooking from '../../assets/icons/eye-looking.svg?raw';
import eyeNotLooking from '../../assets/icons/eye-not-looking.svg?raw';
import share from '../../assets/icons/share.svg?raw';
import more from '../../assets/icons/more.svg?raw';
import puzzlePiece from '../../assets/icons/puzzle-piece.svg?raw';
import send from '../../assets/icons/send.svg?raw';
import key from '../../assets/icons/key.svg?raw';
import usage from '../../assets/icons/usage.svg?raw';
import trash from '../../assets/icons/trash.svg?raw';
import { dev, building } from '$app/environment';

// Newly added icons
import attach from '../../assets/icons/attach.svg?raw';
import bot from '../../assets/icons/bot.svg?raw';
import bug from '../../assets/icons/bug.svg?raw';
import checkCircle from '../../assets/icons/check-circle.svg?raw';
import code from '../../assets/icons/code.svg?raw';
import download from '../../assets/icons/download.svg?raw';
import human from '../../assets/icons/human.svg?raw';
import image from '../../assets/icons/image.svg?raw';
import list from '../../assets/icons/list.svg?raw';
import lockClose from '../../assets/icons/lock-close.svg?raw';
import lockOpen from '../../assets/icons/lock-open.svg?raw';
import mailRead from '../../assets/icons/mail-read.svg?raw';
import mailUnread from '../../assets/icons/mail-unread.svg?raw';
import minusSquare from '../../assets/icons/minus-square.svg?raw';
import plusCircle from '../../assets/icons/plus-circle.svg?raw';
import plusSquare from '../../assets/icons/plus-square.svg?raw';
import search from '../../assets/icons/search.svg?raw';
import upload from '../../assets/icons/upload.svg?raw';
import trophy from '../../assets/icons/trophy.svg?raw';
import menuHamburger from '../../assets/icons/menu-hamburger.svg?raw';
import laptop from '../../assets/icons/laptop.svg?raw';


/**
 * Kepçe Icon Module - Adwaita-style inline SVG icons.
 * GNOME symbolic icon aesthetic: 16×16 grid, monochrome, stroke-based.
 */

export const icons = {
  dice,
  // Navigation / UI
  voteUp,
  voteUpFilled,
  voteDown,
  voteDownFilled,
  chevronDown,
  chevronUp,
  chevronLeft,
  chat,
  warning,
  strongLanguage,
  info,
  'info-critical': infoCritical,
  user,
  profile: user, // Standardized: profile uses user icon
  settings,
  logout,
  'log-out': logout,
  laptop,
  login,
  moon,
  sun,
  system,
  externalLink,
  logo,
  logoSmall,
  logoExperimental,
  logoSmallExperimental,
  verified,
  star,
  starFilled,
  starFilledHalfLeft,
  starFilledHalfRight,
  menuMissing,
  close: crossSmall,
  avatarEmpty,
  calendar,
  noConnection,
  votedUpMore,
  votedDownMore,
  votedEqual,
  votedNone,
  chevronRight,
  puzzle: puzzlePiece,
  cards,
  check,
  wheat,
  utensils,
  keyboard,
  ghost,
  timeout,
  creditCard,
  server,
  bread,
  merge: combine,
  split,
  edit,
  soup,
  meat,
  fish,
  egg,
  dessert,
  tea,
  garlic,
  pepper,
  nut,
  salami,
  commentNone,
  eye: eyeLooking,
  eyeOff: eyeNotLooking,
  share,
  more,
  send,
  key,
  usage,
  trash,

  // Newly Added / Updated
  attach,
  bot,
  bug,
  'check-circle': checkCircle,
  checkCircle,
  code,
  download,
  human,
  image,
  list,
  'lock-close': lockClose,
  'lock-open': lockOpen,
  lock: lockClose, // Fallback for code using 'lock'
  'mail-read': mailRead,
  'mail-unread': mailUnread,
  mail: mailUnread, // Fallback for code using 'mail'
  'minus-square': minusSquare,
  minus: minusSquare, // Fallback for code using 'minus'
  'plus-circle': plusCircle,
  plusCircle, // Fallback for camelCase usage
  'plus-square': plusSquare,
  plus: plusSquare, // Fallback for code using 'plus'
  search,
  upload,
  trophy,
  menuHamburger,
  alert: warning, // Fallback for code using 'alert' (maps to warning)
  alertTriangle: warning,
  'alert-triangle': warning,
  tag: info,
  refresh: chevronLeft,
  'refresh-cw': chevronLeft, // Fallback for refresh-cw
  'rotate-ccw': chevronLeft,
  arrowLeft: chevronLeft, // Fallback for code using 'arrowLeft' (maps to chevronLeft)
  arrowRight: chevronRight, // Fallback for code using 'arrowRight' (maps to chevronRight)
  'arrow-left': chevronLeft,
  'arrow-right': chevronRight,
  'chevron-left': chevronLeft,
  'chevron-right': chevronRight,
  'chevron-up': chevronUp,
  'chevron-down': chevronDown,
  'external-link': externalLink,
  wifiOff: noConnection,
  'wifi-off': noConnection,
  error: infoCritical,
  inbox: mailUnread,
  archive: calendar,
  folder: cards,
  box: cards,
  takeaway: bread,
  slash: warning,
  bell: mailUnread,
  eyeSlash: eyeNotLooking,
  'eye-slash': eyeNotLooking,
  grid: cards,
  layers: list,
  timeline: cards,
};

/**
 * Get an icon SVG string by name.
 * @param {string} name - Icon name from the icons object
 * @param {number} [size=16] - Size override
 * @returns {string} SVG HTML string
 */
export function icon(name, size, label = '') {
  let svg = icons[name] || '';
  if (!svg) {
    if (dev || building) {
      console.warn(`[Kepçe Uyarı] Tanımsız ikon çağrıldı: "${name}"`);
    }
    return '';
  }

  // If size is provided, we force the dimensions. 
  // Otherwise, we respect the SVG's original dimensions (important for logos).
  if (size) {
    if (svg.includes('width=')) {
      svg = svg.replace(/width=["']\d+(px)?["']/, `width="${size}"`);
    } else {
      svg = svg.replace('<svg', `<svg width="${size}"`);
    }

    if (svg.includes('height=')) {
      svg = svg.replace(/height=["']\d+(px)?["']/, `height="${size}"`);
    } else {
      svg = svg.replace('<svg', `<svg height="${size}"`);
    }
  }

  // Handle accessibility and ARIA roles
  if (label) {
    // If it has a label, it's an image that should be announced
    svg = svg.replace('<svg', `<svg role="img" aria-label="${label}"`);
  } else if (!svg.includes('aria-hidden')) {
    // Decorative icons should be hidden from screen readers
    svg = svg.replace('<svg', `<svg aria-hidden="true"`);
  }

  return svg;
}
