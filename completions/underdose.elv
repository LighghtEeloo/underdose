
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
            cand -c 'Use a custom config file'
            cand --config 'Use a custom config file'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Initialize on a new machine, working from drugstore repo'
            cand config 'Configure the machine'
            cand where 'Shows all path information available'
            cand drain 'Drain the machine to the drugstore'
            cand sync 'Drain the machine to the drugstore, and pour on condition'
            cand pour 'Pour the drugstore to the machine'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'underdose;init'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;where'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;drain'= {
            cand -s 'The name of the drugstore repo'
            cand --store 'The name of the drugstore repo'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;sync'= {
            cand -s 'The name of the drugstore repo'
            cand --store 'The name of the drugstore repo'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;pour'= {
            cand -s 'The name of the drugstore repo'
            cand --store 'The name of the drugstore repo'
            cand -f 'f'
            cand --force 'force'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;help'= {
            cand init 'Initialize on a new machine, working from drugstore repo'
            cand config 'Configure the machine'
            cand where 'Shows all path information available'
            cand drain 'Drain the machine to the drugstore'
            cand sync 'Drain the machine to the drugstore, and pour on condition'
            cand pour 'Pour the drugstore to the machine'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'underdose;help;init'= {
        }
        &'underdose;help;config'= {
        }
        &'underdose;help;where'= {
        }
        &'underdose;help;drain'= {
        }
        &'underdose;help;sync'= {
        }
        &'underdose;help;pour'= {
        }
        &'underdose;help;help'= {
        }
    ]
    $completions[$command]
}
