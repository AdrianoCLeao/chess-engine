import { pt } from "./pt";

export type LocaleCode = "pt";

export type Dictionary = typeof pt;

const catalogs: Record<LocaleCode, Dictionary> = {
  pt
};

let current: LocaleCode = "pt";

export function setLocale(code: LocaleCode) {
  current = code in catalogs ? code : "pt";
  document.documentElement.lang = code === "pt" ? "pt-BR" : code;
}

export function getLocale(): LocaleCode {
  return current;
}

export function t(): Dictionary {
  return catalogs[current];
}

/** Interpolate `{name}` placeholders. */
export function format(template: string, vars: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    vars[key] !== undefined ? String(vars[key]) : `{${key}}`
  );
}
