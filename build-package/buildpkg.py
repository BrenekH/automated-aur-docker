#!/bin/env python3
import json, os, tempfile, shutil, subprocess, sys
from pathlib import Path
from typing import List, Tuple

def build(pkg_dir_str: str) -> Tuple[str, bool]:
	"""Build a package and run namcap against the PKGBUILD and the resulting .pkg.tar.zst

	Args:
		pkg_dir_str (str): The directory to read the .aurmanifest.json from

	Returns:
		Tuple[str, bool]: The results of the command as a string and a boolean that represents the failure state (True = failed, False = successful)
	"""
	pkg_dir = Path(pkg_dir_str)

	with (pkg_dir / ".aurmanifest.json").open("r") as f:
		manifest = json.load(f)

	# Stores namcap output for each .pkg.tar.zst created by makepkg
	pkg_file_output = {}

	with tempfile.TemporaryDirectory() as td:
		# Copy PKGBUILD and everything in the manifest.include list to a new directory in /tmp.
		copy_files_to_dir([pkg_dir / "PKGBUILD"] + [pkg_dir / f for f in manifest["include"]], Path(td))

		# TODO: Install AUR dependencies from .aurmanifest.json
		if "aurDeps" in manifest and len(manifest["aurDeps"]) > 0:
			subprocess.run(["/opt/paru", "--mflags", "--skippgpcheck", "--noconfirm", "-Syu"] + [pkg for pkg in manifest["aurDeps"]])

		print("[INFO] Running makepkg")
		makepkg_proc = subprocess.run(["makepkg", "-sm", "--noconfirm", "--noprogressbar"], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

		print("[INFO] Running namcap against PKGBUILD")
		pkgbuild_namcap_proc = subprocess.run(["namcap", "-i", "PKGBUILD"], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

		if makepkg_proc.returncode != 0:
			check_results = create_results_text("makepkg", [("Stdout", makepkg_proc.stdout), ("Stderr", makepkg_proc.stderr)])
			check_results += create_results_text("namcap PKGBUILD", [("Stdout", pkgbuild_namcap_proc.stdout), ("Stderr", pkgbuild_namcap_proc.stderr)])
			check_results += f"Skipping remaining checks: makepkg returned a non-zero exit code {makepkg_proc.returncode}"

			print(f"Skipping remaining checks: makepkg returned a non-zero exit code {makepkg_proc.returncode}")
			return check_results, True

		# Process all built packages
		built_pkg_files = list(Path(td).glob("*.pkg.tar.zst"))

		for built_pkg_file in built_pkg_files:
			print(f"[INFO] Running namcap against {built_pkg_file}")
			pkg_namcap_proc = subprocess.run(["namcap", "-i", str(built_pkg_file)], cwd=td, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)

			print(f"[INFO] Copying built package ({built_pkg_file.name}) to GITHUB_WORKSPACE")
			os.system(f"sudo cp {built_pkg_file} {Path(os.getenv('GITHUB_WORKSPACE')) / built_pkg_file.name}")

			pkg_file_output[built_pkg_file] = pkg_namcap_proc

	check_results = ""
	if makepkg_proc.returncode != 0:
		check_results += create_results_text("makepkg", [("Stdout", makepkg_proc.stdout), ("Stderr", makepkg_proc.stderr)])

	check_results += create_results_text("namcap PKGBUILD", [("Stdout", pkgbuild_namcap_proc.stdout), ("Stderr", pkgbuild_namcap_proc.stderr)])

	pkg_namcap_got_nonzero_exit = False
	for path, process_output in pkg_file_output.items():
		check_results += create_results_text(f"namcap {path.name}", [("Stdout", process_output.stdout), ("Stderr", process_output.stderr)])
		if process_output.returncode != 0:
			pkg_namcap_got_nonzero_exit = True

	return check_results, makepkg_proc.returncode != 0 or pkgbuild_namcap_proc.returncode != 0 or pkg_namcap_got_nonzero_exit

def copy_files_to_dir(files: List[Path], dir: Path):
	for f in files:
		if f.is_absolute():
			print(f"{f} is an absolute Path. It will not be copied.")
			continue
		shutil.copy(f, dir / f.name)

def create_results_text(title: str, pairs: List[Tuple[str, str]]) -> str:
	pair_str = ""

	for pair in pairs:
		pair_str += f"### {pair[0]}:\n```\n{pair[1]}\n```\n"

	return f"""## {title}:\n{pair_str}"""

def set_output(name: str, value: str | bool):
	outputEnvVar = os.getenv("GITHUB_OUTPUT")
	if outputEnvVar == None:
		return

	outputPath = Path(outputEnvVar)

	if isinstance(value, bool):
		value = "true" if value else "false"

	with outputPath.open("a") as f:
		f.write(f"{name}={value}\n")

if __name__ == "__main__":
	output, failed = build(sys.argv[1])

	if "--normal" in sys.argv:
		print(output, f"Failed: {failed}")
	else:
		output = output.replace("\n", "\\n").replace('"', '\\"')
		set_output("result", output)
		set_output("failed", failed)

		if failed:
			sys.exit(1)
