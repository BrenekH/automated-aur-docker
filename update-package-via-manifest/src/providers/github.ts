import { IUpdateProvider } from "../index"

interface IManifestData {

}

export class GitHubUpdateProvider implements IUpdateProvider {
	async latestVersion(manifestData: IManifestData): Promise<string | undefined> {
		throw new Error("Method not implemented.")
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
		throw new Error("Method not implemented.")
	}
}
