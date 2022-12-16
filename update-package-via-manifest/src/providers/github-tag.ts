import * as core from "@actions/core"
import * as github from "@actions/github"
import { GitHub } from "@actions/github/lib/utils"

import { IUpdateProvider } from "../index"

interface IManifestData {
	repo: string
}

export class GitHubTagUpdateProvider implements IUpdateProvider {
	private lastTagHTMLLink: string | undefined
	private octokit: InstanceType<typeof GitHub>

	constructor() {
		this.octokit = github.getOctokit(core.getInput("github-token"))
	}

	async latestVersion(manifestData: IManifestData): Promise<string | undefined> {
		const split = manifestData.repo.split("/")
		const owner = split[0]
		const repoName = split[1]

		if (owner === undefined || repoName === undefined) {
			return undefined
		}

		let latestTag: string | undefined;
		try {
			latestTag = await this.getLatestTag(owner, repoName)
		} catch (e: any) {
			core.warning(e)
			return undefined
		}

		if (latestTag === undefined) {
			return undefined
		}

		this.lastTagHTMLLink = `https://github.com/${owner}/${repoName}/releases/tag/${latestTag}`

		return latestTag.replace(/^v/m, "") // Make sure any leading v's are removed
	}

	async updateData(_: IManifestData): Promise<{
		updateChecksums: boolean
		prContent?: string | undefined
		sourceArray?: string[] | undefined
		source_x86_64?: string[] | undefined
		source_i686?: string[] | undefined
		source_aarch64?: string[] | undefined
		source_armv7h?: string[] | undefined
	} | undefined> {
		if (this.lastTagHTMLLink === undefined) {
			return undefined
		}

		return {
			updateChecksums: true,
			prContent: `__GitHub Tag Link:__ [${this.lastTagHTMLLink}](${this.lastTagHTMLLink})`
		}
	}

	async getLatestTag(owner: string, repoName: string): Promise<string | undefined> {
		const resp = await this.octokit.rest.repos.listTags({
			owner: owner,
			repo: repoName,
		})

		if (resp.status !== 200) {
			throw new Error(`invalid HTTP response code: ${resp.status}`)
		}

		if (resp.data.length === 0) {
			throw new Error("received a 0-length array when listing Git tags")
		}

		resp.data.sort()

		if (resp.data[0] === undefined) {
			throw new Error("Something's broken. Don't know what, but it's broken.")
		}

		return resp.data[0].name
	}
}
