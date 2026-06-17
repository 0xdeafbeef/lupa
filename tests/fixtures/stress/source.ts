export interface RecordSource<T> {
  load(id: string): Promise<T>;
}

export class Store<T extends { id: string }> {
  #source: RecordSource<T>;

  constructor(source: RecordSource<T>) {
    this.#source = source;
  }

  async load(id: string): Promise<T> {
    return this.#source.load(id);
  }

  map<R>(items: T[], project: (item: T) => R): R[] {
    const normalize = (item: T) => ({ ...item, id: item.id.trim() });
    return items.map((item) => project(normalize(item)));
  }
}

export function createStore<T extends { id: string }>(source: RecordSource<T>): Store<T> {
  return new Store(source);
}
