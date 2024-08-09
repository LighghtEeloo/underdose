# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_underdose_global_optspecs
	string join \n h/help V/version
end

function __fish_underdose_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_underdose_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_underdose_using_subcommand
	set -l cmd (__fish_underdose_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c underdose -n "__fish_underdose_needs_command" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_needs_command" -s V -l version -d 'Print version'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "init" -d 'Initialize on a new machine, working from drugstore repo'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "conf" -d 'Configure the machine'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "where" -d 'Shows all path information available'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "sync" -d 'Make a dream on the machine, and pour if possible'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "clean" -d 'Clean up backups'
complete -c underdose -n "__fish_underdose_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c underdose -n "__fish_underdose_using_subcommand init" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_using_subcommand conf" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_using_subcommand where" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_using_subcommand sync" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_using_subcommand clean" -s n -l name -d 'name of the backup' -r
complete -c underdose -n "__fish_underdose_using_subcommand clean" -s v -l version -d 'version of the backup, can be a uuid or "all"' -r
complete -c underdose -n "__fish_underdose_using_subcommand clean" -s h -l help -d 'Print help'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "init" -d 'Initialize on a new machine, working from drugstore repo'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "conf" -d 'Configure the machine'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "where" -d 'Shows all path information available'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "sync" -d 'Make a dream on the machine, and pour if possible'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "clean" -d 'Clean up backups'
complete -c underdose -n "__fish_underdose_using_subcommand help; and not __fish_seen_subcommand_from init conf where sync clean help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
