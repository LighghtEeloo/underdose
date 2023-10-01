
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
            cand config 'Configure the machine'
            cand where 'Shows all path information available'
            cand sync 'Make a dream on the machine, and pour if possible'
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
        &'underdose;sync'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'underdose;help'= {
            cand init 'Initialize on a new machine, working from drugstore repo'
            cand config 'Configure the machine'
            cand where 'Shows all path information available'
            cand sync 'Make a dream on the machine, and pour if possible'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'underdose;help;init'= {
        }
        &'underdose;help;config'= {
        }
        &'underdose;help;where'= {
        }
        &'underdose;help;sync'= {
        }
        &'underdose;help;help'= {
        }
    ]
    $completions[$command]
}
