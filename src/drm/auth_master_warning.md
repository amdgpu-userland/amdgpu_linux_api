## Overwriting master
It is possible that other process has snatched master status away via root
permissions
although it would probably result in a panic because it breaks the assumption that
this client is the current master
