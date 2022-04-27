import * as core from "@actions/core"
import * as github from "@actions/github"
import * as path from "path"
import * as fs from "fs"
import { promise as glob } from "glob-promise"

interface IManifest {
	name: string,
	testCmd: string | null,
	include: string[],
	automaticUpdates: {
		type: string,
		repo: string,
		appID: string,
	}
}

async function main() {
	// Identify all packages in the pkgs directory
	const manifests = await glob("pkgs/**/.aurmanifest.json")

	for (const manifestPath of manifests) {
		const pkgbuildPath = path.join(manifestPath.replace("/.aurmanifest.json", ""), "PKGBUILD")

		core.info(manifestPath)

		const manifest = JSON.parse(fs.readFileSync(manifestPath).toString()) as IManifest

		// TODO: Identify which updateProvider to use

		const pkgbuildVersion: string | undefined = getVersionFromPKGBUILD(manifestPath.replace("/.aurmanifest.json", ""))
		if (pkgbuildVersion === undefined) {
			core.warning(`Unable to get package version from PKGBUILD (${manifest.name})`)
			continue
		}

		// TODO: Ask updateProvider what the latest version is

		// TODO: Check if PKGBUILD and latest versions are the same

		// TODO: Check if a PR has already been opened

		// TODO: Ask updateProvider for the update data to act upon

		// TODO: Update PKGBUILD with new version

		// TODO: Update PKGBUILD source arrays with sources (if provided by updateProvider)

		// TODO: Write PKGBUILD changes to disk

		// TODO: Run updpkgsums if requested by updateProvider

		// TODO: Commit and push changes

		// TODO: Open PR with any custom text from updateProvider
	}
}

function getVersionFromPKGBUILD(dir: string): string | undefined {
	const fileContents: string = fs.readFileSync(path.join(dir, "PKGBUILD")).toString()

	const matches = /^pkgver=(.*)$/m.exec(fileContents)
	if (matches === null) {
		return undefined
	}

	return matches[1]
}

try {
	await main()
} catch (error: any) {
	core.setFailed(error)
}
