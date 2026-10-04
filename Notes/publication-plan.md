# CAFramework Public Publication Plan & Repository Hygiene (D-6)

## Overview
Before releasing `caframework` publicly, two historical artifacts must be cleaned from the git history:
1. Historical legacy binaries (such as `build/bin/caterm` ~15MB).
2. Internal developer notes & audit workbooks under `Notes/`.

## Execution Steps for Owner (Prof. Cecep)

### Step 1: Backup `Notes/` to Private Repository
Create a separate private repository `caframework-notes` to archive the audit trails and implementation plans:
```bash
# In a fresh throwaway clone:
git clone /home/cecepazhar/Project/caframework /tmp/caf-notes-export
cd /tmp/caf-notes-export
git filter-repo --path Notes/ --path-rename Notes/:
git remote add origin git@github.com:cecep-azhar/caframework-notes.git
git push -u origin main --force
```

### Step 2: Purge `Notes/` and Legacy Binaries from Main Repository
In a clean clone of `caframework`:
```bash
git clone /home/cecepazhar/Project/caframework /tmp/caf-public-clean
cd /tmp/caf-public-clean
git filter-repo --invert-paths --path Notes/ --path build/bin/caterm
git count-objects -vH
```

### Step 3: Verification
Ensure no binary artifacts remain in history:
```bash
git log --all -- build/bin/caterm
# Must return empty!
```

### Step 4: Push to Public Remote
```bash
git remote set-url origin git@github.com:cecep-azhar/caframework.git
git push origin main --force
```
