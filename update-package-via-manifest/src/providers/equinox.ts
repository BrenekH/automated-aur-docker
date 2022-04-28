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
		prBody?: string | undefined
		sourceArray?: string[] | undefined
		source_x86_64?: string[] | undefined
		source_i686?: string[] | undefined
		source_aarch64?: string[] | undefined
		source_armv7h?: string[] | undefined
	} | undefined> {
		// TODO: Request source urls for all architectures

		return {
			updateChecksums: false,
		}
	}
}
