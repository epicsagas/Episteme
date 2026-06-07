// Guards against: Feature Envy FP, Long Method FP.
// An event emitter operates on its own internal listener map — all state
// access is local. Methods are short and focused.

type Listener = (...args: unknown[]) => void;

/**
 * Simple synchronous event emitter.
 *
 * All methods work with the internal `listeners` map — there is no
 * feature envy because the class owns the data it operates on.
 */
class EventEmitter {
  private listeners: Map<string, Listener[]> = new Map();

  /**
   * Register a listener for the given event.
   * Returns a function that removes the listener when called.
   */
  on(event: string, fn: Listener): () => void {
    const list = this.listeners.get(event) ?? [];
    list.push(fn);
    this.listeners.set(event, list);
    return () => this.off(event, fn);
  }

  /**
   * Remove a previously registered listener.
   */
  off(event: string, fn: Listener): void {
    const list = this.listeners.get(event);
    if (list === undefined) return;
    const index = list.indexOf(fn);
    if (index !== -1) {
      list.splice(index, 1);
    }
    if (list.length === 0) {
      this.listeners.delete(event);
    }
  }

  /**
   * Emit an event, invoking all registered listeners with the given args.
   */
  emit(event: string, ...args: unknown[]): void {
    const list = this.listeners.get(event);
    if (list === undefined) return;
    for (const fn of list) {
      fn(...args);
    }
  }

  /**
   * Remove all listeners for a specific event, or all events if no
   * event name is provided.
   */
  removeAllListeners(event?: string): void {
    if (event !== undefined) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }

  /**
   * Return the number of listeners for a given event.
   */
  listenerCount(event: string): number {
    return this.listeners.get(event)?.length ?? 0;
  }
}

export { EventEmitter, Listener };
