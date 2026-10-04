export type Paths<T> = T extends object
  ? {
      [K in keyof T]: `${Exclude<K, symbol>}${"" extends Paths<T[K]>
        ? ""
        : `.${Paths<T[K]>}`}`;
    }[keyof T]
  : "";
