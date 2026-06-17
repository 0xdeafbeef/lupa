export class Registry {
  #items = new Map();

  static from(entries) {
    const registry = new Registry();
    entries.forEach(([key, value]) => registry.register(key, value));
    return registry;
  }

  get size() {
    return this.#items.size;
  }

  register(key, factory) {
    function normalize(value) {
      return String(value).trim();
    }
    this.#items.set(normalize(key), () => factory?.() ?? null);
  }

  build(prefix) {
    return [...this.#items.entries()].map(([key, load]) => ({ key: `${prefix}:${key}`, value: load() }));
  }
}

export function createRegistry(entries = []) {
  return Registry.from(entries);
}
