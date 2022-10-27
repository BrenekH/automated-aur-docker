#!/bin/env python3
import json, os, shutil, subprocess, sys
from pathlib import Path
from typing import List, Tuple

def test(pkg_dir_str: str) -> Tuple[str, bool]:
	"""test reads the .aurmanifest.json, and runs the test command

	Args:
		pkg_dir_str (str): The directory to read the .aurmanifest.json from

	Returns:
		Tuple[str, bool]: The results of the command as a string and a boolean that represents the failure state (True = failed, False = successful)
	"""
	pkg_dir = Path(pkg_dir_str)

	with (pkg_dir / ".aurmanifest.json").open("r") as f:
		manifest = json.load(f)

	# Run manifest.testCmd.
	testCmd = None
	if (testCmd := manifest["testCmd"]) != None:
		if type(testCmd) == type(""):
			print("[INFO] Running user-defined testCmd")
			testCmd_proc = subprocess.run(testCmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)
		else:
			print("[ERROR] testCmd must be a string or null")
			sys.exit(1)

	check_results = "## Test Command:\n### Not Run (testCmd is null)"
	if testCmd != None:
		check_results = f"## Test Command:\n### Stdout:\n```\n{testCmd_proc.stdout}\n```\n### Stderr:\n```\n{testCmd_proc.stderr}\n```\n"

	return check_results, testCmd_proc.returncode != 0 if testCmd != None else False

def copy_files_to_dir(files: List[Path], dir: Path):
	for f in files:
		if f.is_absolute():
			print(f"{f} is an absolute Path. It will not be copied.")
			continue
		shutil.copy(f, dir / f.name)

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
	results_out, failed = test(sys.argv[1])

	if "--normal" in sys.argv:
		print(f"{results_out}\n\nFailed: {failed}")
	else:
		results_out = results_out.replace("\n", "\\n").replace('"', '\\"')
		set_output("result", results_out)
		set_output("failed", failed)
		if failed:
			sys.exit(1)
