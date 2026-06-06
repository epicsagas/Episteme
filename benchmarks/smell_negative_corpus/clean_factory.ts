// Guards against: Large Class FP, Switch Statements FP.
// A factory function with a discriminated union switch is the idiomatic way
// to instantiate variants in TypeScript. Each case is a single line.

type AnimalType = "dog" | "cat" | "bird" | "fish" | "hamster";

interface Animal {
  readonly type: AnimalType;
  readonly name: string;
  speak(): string;
}

class Dog implements Animal {
  readonly type = "dog" as const;
  constructor(public readonly name: string) {}
  speak(): string {
    return `${this.name} says woof!`;
  }
}

class Cat implements Animal {
  readonly type = "cat" as const;
  constructor(public readonly name: string) {}
  speak(): string {
    return `${this.name} says meow!`;
  }
}

class Bird implements Animal {
  readonly type = "bird" as const;
  constructor(public readonly name: string) {}
  speak(): string {
    return `${this.name} says tweet!`;
  }
}

class Fish implements Animal {
  readonly type = "fish" as const;
  constructor(public readonly name: string) {}
  speak(): string {
    return `${this.name} says ... (blub blub)`;
  }
}

class Hamster implements Animal {
  readonly type = "hamster" as const;
  constructor(public readonly name: string) {}
  speak(): string {
    return `${this.name} says squeak!`;
  }
}

/**
 * Factory function creating the appropriate Animal subclass.
 *
 * The switch is exhaustive by design — each AnimalType maps to one class.
 * This is not a "switch statement smell" because TypeScript enforces
 * completeness when new types are added.
 */
function createAnimal(type: AnimalType, name: string): Animal {
  switch (type) {
    case "dog":
      return new Dog(name);
    case "cat":
      return new Cat(name);
    case "bird":
      return new Bird(name);
    case "fish":
      return new Fish(name);
    case "hamster":
      return new Hamster(name);
  }
}

export { Animal, AnimalType, createAnimal, Dog, Cat, Bird, Fish, Hamster };
