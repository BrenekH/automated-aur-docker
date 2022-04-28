import * as core from "@actions/core"
import * as github from "@actions/github"

import { IUpdateProvider } from "../index"

interface IManifestData {
	repo: string
}

export class GitHubUpdateProvider implements IUpdateProvider {
	private lastTagHTMLLink: string | undefined

	async latestVersion(manifestData: IManifestData): Promise<string | undefined> {
		const split = manifestData.repo.split("/")
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

		this.lastTagHTMLLink = resp.data.html_url

		return resp.data.tag_name.replace(/^v/m, "") // Make sure any leading v's are removed
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
			prContent: `__GitHub Release Link:__ [${this.lastTagHTMLLink}](${this.lastTagHTMLLink})`
		}
	}
}
