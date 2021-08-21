#!/bin/env python3
import json, tempfile, shutil, subprocess, sys
from pathlib import Path
from typing import List

def test(pkg_dir_str: str) -> bool:
	"""test reads the .aurmanifest.json, and runs the test command

	Args:
		pkg_dir_str (str): The directory to read the .aurmanifest.json from

	Returns:
		bool: Whether or not the test command was successful
	"""
	pkg_dir = Path(pkg_dir_str)

	with (pkg_dir / ".aurmanifest.json").open("r") as f:
		manifest = json.load(f)

	# Run manifest.testCmd.
	testCmd = None
	if (testCmd := manifest["testCmd"]) != None:
		if type(testCmd) == type([]):
			print("[INFO] Running user-defined testCmd")
			testCmd_proc = subprocess.run(testCmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)
		else:
			print("[ERROR] testCmd must be an array or null")
			sys.exit(1)

	check_results = ""
	if testCmd != None and testCmd_proc.returncode != 0:
		check_results += f"testCmd:\nStdout:\n{testCmd_proc.stdout}\nStderr:\n{testCmd_proc.stderr}\n"

	return check_results

def copy_files_to_dir(files: List[Path], dir: Path):
	for f in files:
		if f.is_absolute():
			print(f"{f} is an absolute Path. It will not be copied.")
			continue
		shutil.copy(f, dir / f.name)

if __name__ == "__main__":
	output = test(sys.argv[1])

	if "--normal" in sys.argv:
		print(output)
	else:
		output = output.replace("\n", "\\n").replace('"', '\\"')
		print(f"::set-output name=result::{output}")
