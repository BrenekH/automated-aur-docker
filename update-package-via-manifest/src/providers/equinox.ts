import * as core from "@actions/core"
import axios from "axios"

import { IUpdateProvider } from "../index"

interface IManifestData {
	appID: string
}

export class EquinoxUpdateProvider implements IUpdateProvider {
	async latestVersion(manifestData: IManifestData): Promise<string | undefined> {
		// Send request to equinox.io
		const result = await axios.post("https://update.equinox.io/check", {
			"app_id": manifestData.appID,
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

	async updateData(manifestData: IManifestData): Promise<{
		updateChecksums: boolean
		prContent?: string | undefined
		sourceArray?: string[] | undefined
		source_x86_64?: string[] | undefined
		source_i686?: string[] | undefined
		source_aarch64?: string[] | undefined
		source_armv7h?: string[] | undefined
	} | undefined> {
		const source_x86_64: string | undefined = await this.requestArchSourceURL(manifestData.appID, "amd64")
		const source_i686: string | undefined = await this.requestArchSourceURL(manifestData.appID, "i386")
		const source_aarch64: string | undefined = await this.requestArchSourceURL(manifestData.appID, "arm64")
		const source_armv7h: string | undefined = await this.requestArchSourceURL(manifestData.appID, "arm")

		return {
			updateChecksums: true,
			source_x86_64: (source_x86_64 !== undefined) ? [source_x86_64] : undefined,    // This weird series of ternary statements is all so that
			source_i686: (source_i686 !== undefined) ? [source_i686] : undefined,          // requestArchSourceURL can return a string, but we can
			source_aarch64: (source_aarch64 !== undefined) ? [source_aarch64] : undefined, // also still respect the array-like nature of the
			source_armv7h: (source_armv7h !== undefined) ? [source_armv7h] : undefined,    // source parameters.
		}
	}

	private async requestArchSourceURL(appID: string, arch: string): Promise<string | undefined> {
		const result = await axios.post("https://update.equinox.io/check", {
			"app_id": appID,
			"channel": "stable",
			"current_sha256": "",
			"current_version": "0.0.0",
			"goarm": "",
			"os": "linux",
			"target_version": "",
			"arch": arch,
		})

		if (result.status !== 200) {
			core.warning(`Received non-200 status code from update.equinox.io. Status: ${result.statusText}(${result.status}). Data: ${result.data}`)
			return undefined
		}

		return result.data.download_url
	}
}
