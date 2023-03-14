import { Uri } from "vscode";

export class Dependency {
    constructor(public readonly purl: string) {
    }
}

export class UpdatedDependencies {
    public static readonly NAME = "enforcer/updatedDependencies";

    constructor(
        public readonly root: Uri,
        public readonly dependencies: Array<Dependency>
    ) {
    }
}

export class SeedwingReport {
    constructor(
        public readonly title: string,
        public readonly html: string,
    ) {
    }
}

export class StartOperation {
    public static readonly NAME = "enforcer/startOperation";
    constructor(
        public readonly token: string,
        public readonly title: string,
        public readonly total: number,
    ) {
    }
}

export class UpdateOperation {
    public static readonly NAME = "enforcer/updateOperation";
    constructor(
        public readonly token: string,
        public readonly message: string,
        public readonly increment: number,
    ) {
    }
}

export class FinishOperation {
    public static readonly NAME = "enforcer/finishOperation";
    constructor(
        public readonly token: string,
    ) {
    }
}