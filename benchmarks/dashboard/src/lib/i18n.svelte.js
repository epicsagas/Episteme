export const i18n = $state({ locale: /** @type {'en' | 'ko'} */ ('en') });

export function toggleLocale() {
  i18n.locale = i18n.locale === 'en' ? 'ko' : 'en';
}
