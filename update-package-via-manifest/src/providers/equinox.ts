import * as core from "@actions/core"
import axios from "axios"

import { IUpdateProvider } from "../index"

interface IManifestData {
	appID: string,
	appSlug: string,
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
		const targetArches = [
			["amd64", "x86_64"],
			["386", "i686"],
			["arm64", "aarch64"],
			["arm", "armv7h"],
		];

		const latestVersion = await this.latestVersion(manifestData);
		if (latestVersion === undefined) {
			return undefined;
		}

		let returnObject = {
			updateChecksums: true,
		};

		(await this.getLinuxSourceURLs(manifestData.appSlug, latestVersion))?.map((sourceURL: string) => {
			for (const a of targetArches) {
				if (sourceURL.includes(a[0] as string + ".")) { // Use the beginning of the file extension to ensure arm doesn't match the arm64 url.
					return {
						url: sourceURL,
						arch: a[1] as string,
					};
				}
			}

			return undefined;
		}).forEach((value) => {
			if (value === undefined) { return; }

			returnObject = Object.assign(returnObject, { [`source_${value.arch}`]: [value.url] });
		});

		return returnObject;
	}

	private async getLinuxSourceURLs(appSlug: string, appVersion: string): Promise<string[] | undefined> {
		const version = appVersion.replace(".", "\\.");
		const regex = new RegExp(`https:\\/\\/bin\\.equinox\\.io\\/.*\\/.*-${version}-linux-.*\\.tar\\.gz`, "g");

		const result = await axios.get(`https://dl.equinox.io/${appSlug}/stable/archive`);
		if (result.status !== 200) {
			core.warning(`Received non-200 status code from dl.equinox.io. Status: ${result.statusText}(${result.status}). Data: ${result.data}`);
			return undefined;
		}

		const matches = result.data.match(regex);
		return (matches === null) ? [] : matches;
	}
}
