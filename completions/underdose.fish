complete -c underdose -n "__fish_use_subcommand" -s c -l config -d 'Use a custom config file' -r -F
complete -c underdose -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c underdose -n "__fish_use_subcommand" -f -a "init" -d 'Initialize on a new machine, working from drugstore repo'
complete -c underdose -n "__fish_use_subcommand" -f -a "config" -d 'Configure the machine'
complete -c underdose -n "__fish_use_subcommand" -f -a "where" -d 'Shows all path information available'
complete -c underdose -n "__fish_use_subcommand" -f -a "drain" -d 'Drain the machine to the drugstore'
complete -c underdose -n "__fish_use_subcommand" -f -a "sync" -d 'Drain the machine to the drugstore, and pour on condition'
complete -c underdose -n "__fish_use_subcommand" -f -a "pour" -d 'Pour the drugstore to the machine'
complete -c underdose -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c underdose -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from config" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from where" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from drain" -s s -l store -d 'The name of the drugstore repo' -r
complete -c underdose -n "__fish_seen_subcommand_from drain" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from sync" -s s -l store -d 'The name of the drugstore repo' -r
complete -c underdose -n "__fish_seen_subcommand_from sync" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from pour" -s s -l store -d 'The name of the drugstore repo' -r
complete -c underdose -n "__fish_seen_subcommand_from pour" -s f -l force
complete -c underdose -n "__fish_seen_subcommand_from pour" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize on a new machine, working from drugstore repo'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "config" -d 'Configure the machine'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "where" -d 'Shows all path information available'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "drain" -d 'Drain the machine to the drugstore'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "sync" -d 'Drain the machine to the drugstore, and pour on condition'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "pour" -d 'Pour the drugstore to the machine'
complete -c underdose -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from where; and not __fish_seen_subcommand_from drain; and not __fish_seen_subcommand_from sync; and not __fish_seen_subcommand_from pour; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
