import * as semver from 'semver';

// convert a semver (pre-release) version into a vscode compatible (pre-release) version
//
// In VScode it is possible to flag versions as pre-release. However, they don't support pre-release SemVer tags.
// This means that we have conflicting versions, or need to adopt any VScode versioning scheme (like odd numbers).
//
// Or we generate a version which makes up for it. We simply create a patch version part which is a combination
// of the last two pre-release segments. Assuming a format like `-alpha.1`, we split this into "type" and "number"
// and set the patch version to be originalPath * 100 * 1000 + t * 100 + number.
//
// This gives us a range of 1000 (zero based) numbers for each type, and we define three types (alpha, beta, rc), which
// should be more than enough. Adding the original patch version on top, we have an incrementing representation.

const version = process.env.VERSION || process.argv[2];

const v = semver.parse(version);

const TYPES = {
    "alpha": 1,
    "beta": 2,
    "rc": 99,
};

if (v.prerelease.length > 0) {
    if (v.prerelease.length != 2) {
        throw new Error(`Pre-release must have two components, was: ${v.prerelease}`);
    }

    const t = TYPES[v.prerelease[0]];
    if (t === undefined) {
        throw new Error(`Unknown pre-release type. known types: ${TYPES}, provided: ${v.prerelease[0]}`);
    }

    const n = v.prerelease[1];
    if (typeof n === "string") {
        throw new Error(`Second pre-release component must be numeric, was: ${n}`);
    }
    if (n > 999) {
        throw new Error(`Second pre-release component must be less than 1000, was: ${n}`);
    }

    v.patch = (v.patch * 100 * 1000) + (t * 1000) + n;
    v.prerelease = [];
}

console.log(v.format());