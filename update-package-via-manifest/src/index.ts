import * as core from "@actions/core"
import * as github from "@actions/github"
import { promise as glob } from "glob-promise"
import * as fs from "fs"
import * as path from "path"
import { execSync } from "child_process"
import axios from "axios"

try {
	const context = github.context

	// Identify all packages in the pkgs directory
	const manifests = await glob("pkgs/**/.aurmanifest.json")

	for (const manifestPath of manifests) {
		const pkgbuildPath = path.join(manifestPath.replace("/.aurmanifest.json", ""), "PKGBUILD")

		core.info(manifestPath)

		const manifest = JSON.parse(fs.readFileSync(manifestPath).toString()) as IManifest

		const pkgbuildVersion: string | undefined = getVersionFromPKGBUILD(manifestPath.replace("/.aurmanifest.json", ""))
		if (pkgbuildVersion === undefined) {
			core.warning(`Unable to get package version from PKGBUILD (${manifest.name})`)
			continue
		}

		let latestVersion = ""
		switch (manifest.automaticUpdates.type) {
			case "github":
				const ghVersion = await getLatestVersionFromGithub(manifest.automaticUpdates.repo)
				if (ghVersion === undefined) {
					core.warning(`Failed to get latest version from GitHub (${manifestPath}).`)
					continue
				}
				latestVersion = ghVersion.replace(/^v/m, "") // Remove leading v idiom
				break
			case "equinox":
				const eqVer = await getLatestVersionFromEquinox(manifest.automaticUpdates.appID)
				if (eqVer === undefined) {
					core.warning(`Failed to get latest version from Equinox (${manifestPath}).`)
					continue
				}
				latestVersion = eqVer
				break
			case undefined: // Skip if the automaticUpdates field is empty
				continue
			default:
				core.warning(`Unknown automaticUpdates type '${manifest.automaticUpdates.type}' in ${manifestPath}`)
				continue
		}

		core.info(`PKGBUILD: '${pkgbuildVersion}' Latest: '${latestVersion}'`)
		if (pkgbuildVersion === latestVersion) {
			continue
		}

		// Mark workspace as a safe directory for git operations
		// execSync("git config --global --add safe.directory /github/workspace")

		// Change permissions so that everything "should be" writable
		execSync("sudo chown -R builder:builder $(pwd)", { stdio: 'inherit' })

		core.info(execSync("git remote -v").toString())

		// Check current branches for latestVersion
		if (hasVersionAlreadyBeenPushed(manifest.name, latestVersion)) {
			continue
		}

		// Create new branch
		const branchName = `bot/${manifest.name}/${latestVersion}`
		execSync(`git checkout -b ${branchName}`, { stdio: 'inherit' })

		let pkgbuildContents: string = fs.readFileSync(pkgbuildPath).toString()

		// Update PKGBUILD with new version (pkgver to latestVersion and pkgrel to 1)
		pkgbuildContents = pkgbuildContents.replace(/^pkgver=.*/m, `pkgver=${latestVersion}`)
		pkgbuildContents = pkgbuildContents.replace(/^pkgrel=.*/m, "pkgrel=1")

		fs.writeFileSync(pkgbuildPath, pkgbuildContents)

		// Only update the checksums in PKGBUILD when the manifest type supports it.
		//
		// For example, GitHub urls will update cleanly with the package version, but
		// equinox.io uses random strings in their urls that have to be updated manually.
		// To prevent errors from updpkgsums, we skip the checksum update and rely on
		// maintainers to do it properly.
		if (["github"].indexOf(manifest.automaticUpdates.type) !== -1) {
			// Update checksums in PKGBUILD
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
			...context.repo,
			head: branchName,
			base: "master",
			maintainer_can_modify: true,
			title: `Update ${manifest.name} to ${latestVersion}`,
		})

		// Switch back to master branch
		execSync(`git checkout master`, { stdio: 'inherit' })
	}

} catch (error: any) {
	core.setFailed(error)
}

function getVersionFromPKGBUILD(dir: string): string | undefined {
	const fileContents: string = fs.readFileSync(path.join(dir, "PKGBUILD")).toString()

	const matches = /^pkgver=(.*)$/m.exec(fileContents)
	if (matches === null) {
		return undefined
	}

	return matches[1]
}

async function getLatestVersionFromGithub(repo: string): Promise<string | undefined> {
	const split = repo.split("/")
	const owner = split[0]
	const repoName = split[1]

	if (owner === undefined || repoName === undefined) {
		return undefined
	}

	const octokit = github.getOctokit(core.getInput("github-token"))

	let resp;
	try {
		resp = await octokit.rest.repos.getLatestRelease({
			owner: owner,
			repo: repoName,
		})
	} catch (e: any) {
		core.warning(e)
		return undefined
	}

	if (resp.status !== 200) {
		return undefined
	}

	return resp.data.tag_name
}

async function getLatestVersionFromEquinox(appID: string): Promise<string | undefined> {
	// Send request to equinox.io
	const result = await axios.post("https://update.equinox.io/check", {
		"app_id": appID,
		"channel": "stable",
		"current_sha256": "",
		"current_version": "0.0.0",
		"goarm": "",
		"os": "linux",
		"target_version": "",
		"arch": "amd64"
	})

	if (result.status !== 200) {
		core.warning(`Received non-200 status code from update.equinox.io. Status: ${result.statusText}(${result.status}). Data: ${result.data}`)
		return undefined
	}

	return result.data.release.version
}

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

function hasVersionAlreadyBeenPushed(packageName: string, version: string): boolean {
	const remoteBranches: string = execSync("git ls-remote --heads origin").toString()

	for (const branch of remoteBranches.split("\n")) {
		if (branch.indexOf(`${packageName}/${version}`) === -1) {
			continue
		}
		return true
	}

	return false
}
