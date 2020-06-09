```
lev-ztarter 0.1.0
A. Carscadden
Generate ztarter levs fr elma

USAGE:
    lev-ztarter.exe [FLAGS] [OPTIONS]

FLAGS:
    -g, --generate    generate levels using loaded database(s)
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -r, --from-database <file>         load levs from a database
    -R, --from-databases <files>       load levs from multiple databases
    -d, --from-directory <dir>         load levs from a directory
    -o, --generate-directory <dir>     generate levels to this directory
    -N, --level-amount <num>           amount of levels to generate
    -n, --level-name <name>            name of the level (e.g. for abc123 put abc)
    -p, --level-name-pad <num>         number of padding zeros (e.g. for abc001 put 3)
    -O, --level-number-offset <num>    start numbering levels at this number
        --tag-database <tag>           tag the database that is being written
    -w, --to-database <file>           write database
```