
use builtin;
use str;

set edit:completion:arg-completer[underdose] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'underdose'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'underdose'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Initialize on a new machine, working from drugstore repo'
            cand conf 'Configure the machine'
            cand where 'Shows all path information available'
            cand sync 'Make a dream on the machine, and pour if possible'
            cand clean 'Clean up backups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'underdose;init'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;conf'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;where'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;sync'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;clean'= {
            cand -n 'name of the backup'
            cand --name 'name of the backup'
            cand -v 'version of the backup, can be a uuid or "all"'
            cand --version 'version of the backup, can be a uuid or "all"'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;help'= {
            cand init 'Initialize on a new machine, working from drugstore repo'
            cand conf 'Configure the machine'
            cand where 'Shows all path information available'
            cand sync 'Make a dream on the machine, and pour if possible'
            cand clean 'Clean up backups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'underdose;help;init'= {
        }
        &'underdose;help;conf'= {
        }
        &'underdose;help;where'= {
        }
        &'underdose;help;sync'= {
        }
        &'underdose;help;clean'= {
        }
        &'underdose;help;help'= {
        }
    ]
    $completions[$command]
}
