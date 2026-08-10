export const setCache = (key: string, value: any) => {
  value = typeof value === "string" ? value : JSON.stringify(value);
  window.localStorage && window.localStorage.setItem(key, value);
};

export const getCache = (key: string, defaultVal: any = null): any => {
  let val = defaultVal;
  try {
    val = window.localStorage && window.localStorage.getItem(key);
    if (val === null) {
      return defaultVal;
    }
    if (val) {
      const parseVal = JSON.parse(val);
      if (parseVal !== null) {
        return parseVal;
      }
    }
    return val;
  } catch (error) {
    return val;
  }
};

export const removeCache = (key: string) => {
  window.localStorage && window.localStorage.removeItem(key);
};
