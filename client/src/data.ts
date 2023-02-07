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