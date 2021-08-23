#!/bin/env python3
import json, os, tempfile, shutil, subprocess, sys
from pathlib import Path
from typing import List, Tuple

def build(pkg_dir_str: str) -> Tuple[str, bool]:
	"""build builds a package and runs namcap against the PKGBUILD and the resulting .pkg.tar.zst

	Args:
		pkg_dir_str (str): The directory to read the .aurmanifest.json from

	Returns:
		Tuple[str, bool]: The results of the command as a string and a boolean that represents the failure state (True = failed, False = successful)
	"""
	pkg_dir = Path(pkg_dir_str)

	with (pkg_dir / ".aurmanifest.json").open("r") as f:
		manifest = json.load(f)

	with tempfile.TemporaryDirectory() as td:
		# Copy PKGBUILD and everything in the manifest.include list to a new directory in /tmp.
		copy_files_to_dir([pkg_dir / "PKGBUILD"] + [pkg_dir / f for f in manifest["include"]], Path(td))

		print("[INFO] Running makepkg")
		makepkg_proc = subprocess.run(["makepkg", "-sm", "--noconfirm", "--noprogressbar"], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

		print("[INFO] Running namcap against PKGBUILD")
		pkgbuild_namcap_proc = subprocess.run(["namcap", "-i", "PKGBUILD"], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

		if makepkg_proc.returncode != 0:
			check_results = f"makepkg:\nStdout:\n{makepkg_proc.stdout}\nStderr:\n{makepkg_proc.stderr}\n"
			check_results += f"namcap PKGBUILD:\nStdout:\n{pkgbuild_namcap_proc.stdout}\nStderr:\n{pkgbuild_namcap_proc.stdout}\n"
			check_results += f"Skipping remaining checks: makepkg returned a non-zero exit code {makepkg_proc.returncode}"
			print(f"Skipping remaining checks: makepkg returned a non-zero exit code {makepkg_proc.returncode}")
			return check_results, True

		# Find the built package
		built_pkg_file = str(list(Path(td).glob("*.pkg.tar.zst"))[0])

		print("[INFO] Running namcap against generated package")
		pkg_namcap_proc = subprocess.run(["namcap", "-i", built_pkg_file], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

		print("[INFO] Copying built package to GITHUB_WORKSPACE")
		os.system(f"sudo cp {built_pkg_file} {Path(os.getenv('GITHUB_WORKSPACE')) / 'package.pkg.tar.zst'}")

	check_results = ""
	if makepkg_proc.returncode != 0:
		check_results += f"makepkg:\nStdout:\n{makepkg_proc.stdout}\nStderr:\n{makepkg_proc.stderr}\n"

	check_results += f"namcap PKGBUILD:\nStdout:\n{pkgbuild_namcap_proc.stdout}\nStderr:\n{pkgbuild_namcap_proc.stdout}\n"
	check_results += f"namcap *.pkg.tar.zst:\nStdout\n{pkg_namcap_proc.stdout}\nStderr:\n{pkgbuild_namcap_proc.stderr}\n"

	return check_results, makepkg_proc.returncode != 0 or pkgbuild_namcap_proc.returncode != 0 or pkg_namcap_proc.returncode != 0

def copy_files_to_dir(files: List[Path], dir: Path):
	for f in files:
		if f.is_absolute():
			print(f"{f} is an absolute Path. It will not be copied.")
			continue
		shutil.copy(f, dir / f.name)

if __name__ == "__main__":
	output, failed = build(sys.argv[1])

	if "--normal" in sys.argv:
		print(output, f"Failed: {failed}")
	else:
		output = output.replace("\n", "\\n").replace('"', '\\"')
		print(f"::set-output name=result::{output}")
		print(f"::set-output name=failed::{failed}")

		if failed:
			sys.exit(1)
