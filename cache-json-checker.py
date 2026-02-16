import json
import zipfile
import os

# Get the directory of this script
BASE_DIR = os.path.dirname(os.path.abspath(__file__))

file = os.path.join(BASE_DIR, ".tmp", "cache.json")
submissions_zip = os.path.join(BASE_DIR, ".tmp", "submissions.zip")

# ===== Part 1: Load JSON array and find duplicates =====
passed_test_duplicate_not_found= True
seen = set()

with open(file, 'r') as f:
	data = json.load(f)

arr = data["submission-file-with-no-tickers"]

seen = set()
duplicates = set()
for x in arr:
	if x in seen:
		passed_test_duplicate_not_found = False
		print(f"Duplicate found: {x}")
		duplicates.add(x)
	else:
		seen.add(x)

print("passed_test_duplicate_not_found (Expecting True):", passed_test_duplicate_not_found)
print("Seen:", len(seen))
print("Duplicates:", duplicates)

# ===== Part 2: Load JSON files from zip and check tickers =====
passed_test_no_empty_tickers_array_found = False

with zipfile.ZipFile(submissions_zip, 'r') as zipf:
	for filename in arr:
		if filename.endswith(".json"):
			try:
				with zipf.open(filename) as f:
					submission_data = json.load(f)
			except KeyError:
				print(f"File {filename} not found in zip.")
				continue

			tickers = submission_data.get("tickers", [])
			if tickers:
				passed_test_no_empty_tickers_array_found = True
				print(f"Tickers array has {len(tickers)} items in file: {filename}")

print("passed_test_no_empty_tickers_array_found (Expecting True):", passed_test_no_empty_tickers_array_found)

print("Check complete.")
