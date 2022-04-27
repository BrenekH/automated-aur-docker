import * as core from "@actions/core"
import * as github from "@actions/github"
import * as path from "path"
import * as fs from "fs"
import { execSync } from "child_process"
import { promise as glob } from "glob-promise"

import * as util from "./util"

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

	const pkgbuildVersion: string | undefined = util.getVersionFromPKGBUILD(manifestPath.replace("/.aurmanifest.json", ""))
	if (pkgbuildVersion === undefined) {
		core.warning(`Unable to get package version from PKGBUILD (${manifest.name})`)
		return
	}

	const latestVersion = updProv.latestVersion(manifest.automaticUpdates)

	core.info(`PKGBUILD: '${pkgbuildVersion}' Latest: '${latestVersion}'`)

	if (pkgbuildVersion === latestVersion) return

	// Change permissions so that everything "should be" writable and so git won't complain
	// about an unsafe directory
	execSync("sudo chown -R builder:builder $(pwd)", { stdio: 'inherit' })

	// Check current branches for latestVersion
	if (util.hasVersionAlreadyBeenPushed(manifest.name, latestVersion)) return

	const updateData = updProv.updateData(manifest.automaticUpdates)

	// Create new branch so that a PR can be made later
	const branchName = `bot/${manifest.name}/${latestVersion}`
	execSync(`git checkout -b ${branchName}`, { stdio: 'inherit' })

	let pkgbuildContents: string = fs.readFileSync(pkgbuildPath).toString()

	// Update PKGBUILD with new version (pkgver to latestVersion and pkgrel to 1)
	pkgbuildContents = pkgbuildContents.replace(/^pkgver=.*/m, `pkgver=${latestVersion}`)
	pkgbuildContents = pkgbuildContents.replace(/^pkgrel=.*/m, "pkgrel=1")

	// Update PKGBUILD source arrays with new sources (if provided by updateProvider)
	pkgbuildContents = util.updateSourceArrays(pkgbuildContents, updateData)

	fs.writeFileSync(pkgbuildPath, pkgbuildContents)

	// TODO: Run updpkgsums if requested by updateProvider

	// TODO: Commit and push changes

	// TODO: Open PR with any custom text from updateProvider
}

try {
	await main()
} catch (error: any) {
	core.setFailed(error)
}
