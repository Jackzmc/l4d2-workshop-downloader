# l4d2-workshop-downloader

A work in progress CLI tool that manages workshop items, as the current workshop implementation in L4D2 can be annoyingly slow and broken. Written as my first rust program.

Current features:
[x] Import Workshop VPKs
[ ] Update existing VPKs
[ ] Search for items
[ ] Manage existing items

Currently import workshop vpks is the only working option which will take the vpks in your workshop folder, find the names and move them to your addons folder, to then be updated at a later date using a meta file (downloads.json). Currently the filepaths are hardcoded at this time. 

In the future you will put the exe in your addons folder.