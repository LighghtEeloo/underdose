
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'underdose' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'underdose'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'underdose' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize on a new machine, working from drugstore repo')
            [CompletionResult]::new('conf', 'conf', [CompletionResultType]::ParameterValue, 'Configure the machine')
            [CompletionResult]::new('where', 'where', [CompletionResultType]::ParameterValue, 'Shows all path information available')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Make a dream on the machine, and pour if possible')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Clean up backups')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'underdose;init' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'underdose;conf' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'underdose;where' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'underdose;sync' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'underdose;clean' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'name of the backup')
            [CompletionResult]::new('--name', 'name', [CompletionResultType]::ParameterName, 'name of the backup')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'version of the backup, can be a uuid or "all"')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'version of the backup, can be a uuid or "all"')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'underdose;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize on a new machine, working from drugstore repo')
            [CompletionResult]::new('conf', 'conf', [CompletionResultType]::ParameterValue, 'Configure the machine')
            [CompletionResult]::new('where', 'where', [CompletionResultType]::ParameterValue, 'Shows all path information available')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Make a dream on the machine, and pour if possible')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Clean up backups')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'underdose;help;init' {
            break
        }
        'underdose;help;conf' {
            break
        }
        'underdose;help;where' {
            break
        }
        'underdose;help;sync' {
            break
        }
        'underdose;help;clean' {
            break
        }
        'underdose;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
