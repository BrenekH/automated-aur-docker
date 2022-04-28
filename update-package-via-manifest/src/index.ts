import * as core from "@actions/core"
import * as github from "@actions/github"
import * as path from "path"
import * as fs from "fs"
import { execSync } from "child_process"
import { promise as glob } from "glob-promise"

import * as util from "./util"
import { GitHubUpdateProvider } from "./providers/github"
import { EquinoxUpdateProvider } from "./providers/equinox"

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

export interface IUpdateProvider {
	latestVersion(manifestData: any): Promise<string | undefined>,
	updateData(manifestData: any): Promise<{
		updateChecksums: boolean,
		prContent?: string,
		sourceArray?: Array<string>,
		source_x86_64?: Array<string>,
		source_i686?: Array<string>,
		source_aarch64?: Array<string>,
		source_armv7h?: Array<string>,
	} | undefined>,
}

const updateProviders: Map<string, IUpdateProvider> = new Map<string, IUpdateProvider>([
	["github", new GitHubUpdateProvider()],
	["equinox", new EquinoxUpdateProvider()],
]);

async function main() {
	// Identify all packages in the pkgs directory
	const manifests = await glob("pkgs/**/.aurmanifest.json")

	for (const manifestPath of manifests) {
		const pkgbuildPath = path.join(manifestPath.replace("/.aurmanifest.json", ""), "PKGBUILD")

		core.info(manifestPath)

		await handleManifest(manifestPath, pkgbuildPath)
	}
}

async function handleManifest(manifestPath: string, pkgbuildPath: string) {
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

	const latestVersion = await updProv.latestVersion(manifest.automaticUpdates)
	core.info(`PKGBUILD: '${pkgbuildVersion}' Latest: '${latestVersion}'`)

	if (latestVersion === undefined) return

	if (pkgbuildVersion === latestVersion) return

	// Change permissions so that everything "should be" writable and so git won't complain
	// about an unsafe directory
	execSync("sudo chown -R builder:builder $(pwd)", { stdio: 'inherit' })

	// Check current branches for latestVersion
	if (util.hasVersionAlreadyBeenPushed(manifest.name, latestVersion)) return

	const updateData = await updProv.updateData(manifest.automaticUpdates)

	if (updateData === undefined) {
		core.warning(`Cannot operate on undefined data from ${manifest.automaticUpdates.type} update provider. Skipping ${manifestPath}`)
		return
	}

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

	// Update checksums in PKGBUILD
	if (updateData.updateChecksums) {
		execSync(`updpkgsums`, {
			stdio: "inherit",
			cwd: manifestPath.replace("/.aurmanifest.json", ""),
		})
	}

	// git add and commit updated PKGBUILD
	execSync(`git add ${pkgbuildPath}`, { stdio: 'inherit' })
	execSync(`git commit -m "Update ${manifest.name} to ${latestVersion}"`, {
		stdio: "inherit",
		env: {
			GIT_AUTHOR_NAME: "github-actions[bot]",
			GIT_AUTHOR_EMAIL: "41898282+github-actions[bot]@users.noreply.github.com",
			GIT_COMMITTER_NAME: "github-actions[bot]",
			GIT_COMMITTER_EMAIL: "41898282+github-actions[bot]@users.noreply.github.com",
			EMAIL: "41898282+github-actions[bot]@users.noreply.github.com",
		}
	})

	// Push changes to GitHub
	execSync(`git push origin ${branchName}`, { stdio: 'inherit' })

	// Create a pull request with the changes
	const octokit = github.getOctokit(core.getInput("github-token"))
	octokit.rest.pulls.create({
		...github.context.repo,
		head: branchName,
		base: "master",
		maintainer_can_modify: true,
		title: `Update ${manifest.name} to ${latestVersion}`,
		body: `${(updateData.prContent !== undefined) ? updateData.prContent + "\n\n" : ""}_This PR was automatically opened by the [Automatic AUR system](https://github.com/BrenekH/automated-aur#README)._`,
	})

	// Switch back to master branch
	execSync(`git checkout master`, { stdio: 'inherit' })
}

try {
	await main()
} catch (error: any) {
	core.setFailed(error)
}
