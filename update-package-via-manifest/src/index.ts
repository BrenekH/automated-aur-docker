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
	},
}

interface IUpdateProvider {
	latestVersion(manifestData: any): string,
	updateData(manifestData: any): {
		updateChecksums: boolean,
		sourceArray?: Array<string>,
		source_x86_64?: Array<string>,
		source_i686?: Array<string>,
		source_aarch64?: Array<string>,
		source_armv7h?: Array<string>,
	},
}

const updateProviders: Map<string, IUpdateProvider> = new Map<string, IUpdateProvider>();

async function main() {
	// Identify all packages in the pkgs directory
	const manifests = await glob("pkgs/**/.aurmanifest.json")

	for (const manifestPath of manifests) {
		const pkgbuildPath = path.join(manifestPath.replace("/.aurmanifest.json", ""), "PKGBUILD")

		core.info(manifestPath)

		handleManifest(manifestPath, pkgbuildPath)

	}
}

function handleManifest(manifestPath: string, pkgbuildPath: string) {
	const manifest = JSON.parse(fs.readFileSync(manifestPath).toString()) as IManifest

	if (manifest.automaticUpdates === undefined || manifest.automaticUpdates.type === undefined) {
		return
	}

	const updProv = updateProviders.get(manifest.automaticUpdates.type)
	if (updProv === undefined) {
		core.warning(`Unknown automaticUpdates type '${manifest.automaticUpdates.type}' in ${manifestPath}`)
		return
	}

	const pkgbuildVersion: string | undefined = getVersionFromPKGBUILD(manifestPath.replace("/.aurmanifest.json", ""))
	if (pkgbuildVersion === undefined) {
		core.warning(`Unable to get package version from PKGBUILD (${manifest.name})`)
		return
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
