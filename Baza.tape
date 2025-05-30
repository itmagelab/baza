# VHS documentation
#
# Output:
#   Output <path>.gif               Create a GIF output at the given <path>
#   Output <path>.mp4               Create an MP4 output at the given <path>
#   Output <path>.webm              Create a WebM output at the given <path>
#
# Require:
#   Require <string>                Ensure a program is on the $PATH to proceed
#
# Settings:
#   Set FontSize <number>           Set the font size of the terminal
#   Set FontFamily <string>         Set the font family of the terminal
#   Set Height <number>             Set the height of the terminal
#   Set Width <number>              Set the width of the terminal
#   Set LetterSpacing <float>       Set the font letter spacing (tracking)
#   Set LineHeight <float>          Set the font line height
#   Set LoopOffset <float>%         Set the starting frame offset for the GIF loop
#   Set Theme <json|string>         Set the theme of the terminal
#   Set Padding <number>            Set the padding of the terminal
#   Set Framerate <number>          Set the framerate of the recording
#   Set PlaybackSpeed <float>       Set the playback speed of the recording
#   Set MarginFill <file|#000000>   Set the file or color the margin will be filled with.
#   Set Margin <number>             Set the size of the margin. Has no effect if MarginFill isn't set.
#   Set BorderRadius <number>       Set terminal border radius, in pixels.
#   Set WindowBar <string>          Set window bar type. (one of: Rings, RingsRight, Colorful, ColorfulRight)
#   Set WindowBarSize <number>      Set window bar size, in pixels. Default is 40.
#   Set TypingSpeed <time>          Set the typing speed of the terminal. Default is 50ms.
#
# Sleep:
#   Sleep <time>                    Sleep for a set amount of <time> in seconds
#
# Type:
#   Type[@<time>] "<characters>"    Type <characters> into the terminal with a
#                                   <time> delay between each character
#
# Keys:
#   Escape[@<time>] [number]        Press the Escape key
#   Backspace[@<time>] [number]     Press the Backspace key
#   Delete[@<time>] [number]        Press the Delete key
#   Insert[@<time>] [number]        Press the Insert key
#   Down[@<time>] [number]          Press the Down key
#   Enter[@<time>] [number]         Press the Enter key
#   Space[@<time>] [number]         Press the Space key
#   Tab[@<time>] [number]           Press the Tab key
#   Left[@<time>] [number]          Press the Left Arrow key
#   Right[@<time>] [number]         Press the Right Arrow key
#   Up[@<time>] [number]            Press the Up Arrow key
#   Down[@<time>] [number]          Press the Down Arrow key
#   PageUp[@<time>] [number]        Press the Page Up key
#   PageDown[@<time>] [number]      Press the Page Down key
#   Ctrl+<key>                      Press the Control key + <key> (e.g. Ctrl+C)
#
# Display:
#   Hide                            Hide the subsequent commands from the output
#   Show                            Show the subsequent commands in the output

Output contrib/Baza.gif
Output contrib/Baza.mp4

Require baza

Set Shell "bash"
Set WindowBar Colorful
Set Theme "Batman"
Set FontSize 20
Set Width 1024
Set Height 768
Set TypingSpeed 100ms

Type "# Get started with Baza!" Enter Sleep 1s Ctrl+l
Type "# Initialize your new storage..." Enter Sleep 1s Ctrl+l
Type "baza init -p MyPasswordPhrase" Sleep 1s Enter Sleep 3s
Type "# Adding a new password bundle" Sleep 1s Enter Sleep 1s
Type "baza -a work::new::login" Sleep 1s Enter Sleep 2s
Insert Type "My$ecreTPa$$w0rd" Enter Sleep 1s
Enter Type "This is example of description" Sleep 1s
Enter Type "site: https://domain.ltd" Sleep 1s
Escape Type ":wq" Sleep 1s Enter
Type "baza bundle search new" Sleep 1s Enter Sleep 2s
Type "baza --show work::new::login" Sleep 1s Enter Sleep 2s
Type "echo '$eCret' | baza --stdin other::new::password" Sleep 1s Enter Sleep 2s Ctrl+l
Type "# You can lock your Baza for display and editing" Sleep 1s Enter Sleep 1s
Type "baza lock" Sleep 1s Enter Sleep 3s
Type "baza bundle edit work::new::login" Sleep 1s Enter Sleep 3s
Type "baza unlock" Sleep 1s Enter Sleep 3s
Type "MyPasswordPhrase" Sleep 1s Enter Sleep 3s
Type "baza bundle copy work::new::login" Sleep 1s Enter Sleep 2s Ctrl+C Enter
Type "baza bundle delete work::new::login" Sleep 1s Enter Sleep 2s
Type "baza bundle search new" Sleep 1s Enter
Type "baza password generate --help" Sleep 1s Enter Sleep 2s
Type "baza password generate 12 --no-symbols" Sleep 1s Enter Sleep 2s
