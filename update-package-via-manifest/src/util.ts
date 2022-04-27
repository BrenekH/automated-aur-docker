import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process"

export function getVersionFromPKGBUILD(dir: string): string | undefined {
	const fileContents: string = fs.readFileSync(path.join(dir, "PKGBUILD")).toString()

	const matches = /^pkgver=(.*)$/m.exec(fileContents)
	if (matches === null) {
		return undefined
	}

	return matches[1]
}

export function hasVersionAlreadyBeenPushed(packageName: string, version: string): boolean {
	const remoteBranches: string = execSync("git ls-remote --heads origin").toString()

	for (const branch of remoteBranches.split("\n")) {
		if (branch.indexOf(`${packageName}/${version}`) === -1) {
			continue
		}
		return true
	}

	return false
}
