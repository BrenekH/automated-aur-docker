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

export function updateSourceArrays(contents: string, arrays: {
	sourceArray?: Array<string>,
	source_x86_64?: Array<string>,
	source_i686?: Array<string>,
	source_aarch64?: Array<string>,
	source_armv7h?: Array<string>,
}): string {
	if (arrays.sourceArray !== undefined) {
		contents = contents.replace(/^source=\(.*\)/m, `source=${formatAsBashArray(arrays.sourceArray)}`)
	}

	if (arrays.source_x86_64 !== undefined) {
		contents = contents.replace(/^source_x86_64=\(.*\)/m, `source=${formatAsBashArray(arrays.source_x86_64)}`)
	}

	if (arrays.source_i686 !== undefined) {
		contents = contents.replace(/^source_i686=\(.*\)/m, `source=${formatAsBashArray(arrays.source_i686)}`)
	}

	if (arrays.source_aarch64 !== undefined) {
		contents = contents.replace(/^source_aarch64=\(.*\)/m, `source=${formatAsBashArray(arrays.source_aarch64)}`)
	}

	if (arrays.source_armv7h !== undefined) {
		contents = contents.replace(/^source_arm7vh=\(.*\)/m, `source=${formatAsBashArray(arrays.source_armv7h)}`)
	}

	return contents
}

function formatAsBashArray(arr: Array<string>): string {
	let res = ""

	for (const item of arr) {
		res += `"${item}" `
	}

	return `(${res.slice(0, res.length - 1)})`
}
