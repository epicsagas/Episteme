// Guards against: Middle Man FP.
// A caching decorator adds real value (cache lookup/store) around each
// delegated call — the delegation is not mindless forwarding.

interface Repository<T> {
  findById(id: string): Promise<T | null>;
  findAll(): Promise<T[]>;
  save(entity: T): Promise<void>;
  delete(id: string): Promise<void>;
}

interface Cache {
  get(key: string): unknown | undefined;
  set(key: string, value: unknown, ttlMs?: number): void;
  delete(key: string): void;
}

/**
 * Caching decorator that wraps a Repository with a write-through cache.
 *
 * Each method checks the cache first, delegates on miss, and populates
 * the cache on the way back. This is a legitimate decorator pattern,
 * not accidental delegation.
 */
class CachedRepository<T> implements Repository<T> {
  private readonly prefix = "repo";

  constructor(
    private readonly inner: Repository<T>,
    private readonly cache: Cache,
    private readonly ttlMs: number = 60_000,
  ) {}

  async findById(id: string): Promise<T | null> {
    const key = `${this.prefix}:${id}`;
    const cached = this.cache.get(key) as T | undefined;
    if (cached !== undefined) {
      return cached;
    }
    const result = await this.inner.findById(id);
    if (result !== null) {
      this.cache.set(key, result, this.ttlMs);
    }
    return result;
  }

  async findAll(): Promise<T[]> {
    const key = `${this.prefix}:all`;
    const cached = this.cache.get(key) as T[] | undefined;
    if (cached !== undefined) {
      return cached;
    }
    const results = await this.inner.findAll();
    this.cache.set(key, results, this.ttlMs);
    return results;
  }

  async save(entity: T & { id?: string }): Promise<void> {
    await this.inner.save(entity);
    if (entity.id !== undefined) {
      this.cache.delete(`${this.prefix}:${entity.id}`);
    }
    this.cache.delete(`${this.prefix}:all`);
  }

  async delete(id: string): Promise<void> {
    await this.inner.delete(id);
    this.cache.delete(`${this.prefix}:${id}`);
    this.cache.delete(`${this.prefix}:all`);
  }
}

export { CachedRepository, Repository, Cache };
