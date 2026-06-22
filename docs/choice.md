# Choices

There were choices made, some for flexibility, performance, ease of use or development and functionality

## choosing the GUI framework

First i was looking at the QT project, pyside6 then QML then it hit me

I WANT A DECLARATIVE APPROACH (Nix influence is real), so I started using QML and I regret every single moment that has been invested in the framework of QT and all it's stupidly designed relatives / bindings

the reason for that is

- huge dependency graph
- huge bundle size
- very heavy responsiveness
- bundling a whole js renderer (to my understanding this is what it does under the hood I could be wrong so independently double check)

### Resolution

As per declarative approaches for UIs, I found slint and i was immediately blown away, small bundle size, declarative model with a live-view, cross-platform and lightweight on resources!

> basically the best I could ever hope for in my opinion

so, yeah i am happy i discovered a project that will always be of value to me now and the future

## choosing a language

I first settled on python because the first search result when i was developing this back at 2024 is the `piexif` python library so i felt like i was locked and developed the whole application in the sense that this is the only solution

then I tried the QT system which was as said before horrible (or i am the one who doesn't understand QT application development really idk)

2 years later, revisiting the topic armed with a strong sense that there must be another way or something I am missing, that it can't be that there is nothing except a obscure python library not updated 7 years ago at the time of writing (2026), then I found it, I found the most complete thing I will ever find for my application, `exiv2` which thankfully have a utility with the same name that allows execution FROM ANY LANGUAGE

now as there is a working code in python, why rewrite ? duh why not rewrite ??

python is a horrible language in resource consumption and any investment in it just makes you look like a clown (i am looking at you AI bees)

## Did you use AI?

I did, and it's not in the oh a "vibe coder" I am a person with a CS degree,
I understand system and principled enough to review what the AI write so it doesn't mess things up, WHILE being aware i will be deemed responsible as the commits are in my name.

There is nothing wrong with engineering ideas and building systems and offloading (not entirely either) the testing, or coding to AI which was the case for this project, I didn't offload my thinking nor intelligence but I allowed the AI to offload the execution (writing code not using commands on my system haha a funny `rm -rf /` in a distance )

I understand the code and the system and if someone asks a question I can answer them.
