# Jev manual, version 1

A decisions model never reads a manual. It is handed a `state` and questions
whose options carry their own descriptions, and it answers those questions and
nothing else. So **this file is the manual**, and the sections below are
literally what gets sent.

It is **an input under test, not truth.** Its hash goes into every run record,
so two runs are comparable only when they used the same manual, and a lead may
legitimately say "this manual's line about X is what sank the character". That
is the spirit the strategy manual states for its own chapter 15: a starting
policy to be measured.

## instructions

Which single key should be pressed next in this game of Angband 4.2.6?

Read the top line of the screen first. It is the game's message line and it
usually says why the last key did nothing: "I see no up staircase here" means
you are not standing on one. A `-more-` there means a message is waiting and
any key clears it. A question ending in `[y/n]` wants y or n. A menu wants the
letter of a row, or Enter for the highlighted one.

The original keyset is in use, so the digits move and the roguelike keys do not
apply. A key listed as already pressed here without effect will not work this
time either, so choose a different one.

Descending is the point of the game: levels and gear come from depth and
nothing is gained in the town. But descending with no escape and no healing is
how characters die on level 1.

## state

What is sent with every question. This is the contract; nothing else is sent,
and nothing is interpreted for the model.

screen = the 24 lines the game drew, exactly as drawn, trailing blanks removed
turn = the game's own turn counter. It lags through a burst of prompts, because
       the game only counts a turn after a full pass of its loop, so it is not
       a reliable sign of progress
depth_in_levels = the game's own `player->depth`. 0 is the town, 1 is 50 feet
deepest_reached = `player->max_depth`, the deepest level this character reached
cursor = where the game put the cursor. In a menu this is the highlighted row,
         which a person sees as reverse video and the characters cannot show
the_key_just_pressed = the last key sent
the_screen_changed_after_it = whether that key changed the screen at all
keys_already_pressed_here_that_changed_nothing = every key tried while this
         exact screen stayed as it is. A person remembers trying them; a
         decisions model cannot, because it never writes anything down
the_goal_you_chose_last_time = your own previous answer to the goal question.
         This is the whole of the memory available to you, and you wrote it
the_games_own_manual = the pinned manual pages, when the run asks for them
         (`--manual-pages`). Off by default: measured at five times the cost
         and no measurable difference, 2026-11-09

## strategy

Compressed from `docs/c-manual/` at the borg-spec pin, which says of its own
chapter 15 that these are "a starting policy to be measured, not truths".
Rule ids are kept so a lead can name what it is arguing with.

ESC-01 = the escape ladder, in order: standing on a staircase, take it - it is
         one action and certain; else a single dangerous monster near and the
         rest far, read Phase Door; else read or use Teleport; else Teleport
         Level if the whole level must go; else heal, and only as a last
         resort drink something unknown
DEPTH-01 = gate depth on the threat you can actually survive, not on character
         level. The usual death deep is one big hit you do not resist
STAIR-01 = a level that announces danger on arrival ("This place seems
         murderous", "dangerous") and gives you nothing is worth leaving by
         the staircase you arrived on
FIGHT-01 = a fight is worth entering when you can win it before it can kill
         you. At character level 1 with a starting weapon that is almost
         nothing, which is why thrown oil matters
OIL-01 = a thrown flask of oil does far more than an early melee weapon and
         works on most things. Buy ten or more before the first descent; they
         are cheap, and they are the early character's real attack
REST-01 = rest to full before taking a staircase down. One rest command costs
         one decision and hundreds of game turns, and game turns are not the
         scarce thing here
TOWN-01 = the town has no experience in it. Buy escapes first (Phase Door),
         then a heal (Cure Light Wounds), then oil, then go down
ANTI-11 = killing townspeople for money keeps a character at level 1. Do not

## goals

Asked as a second question on every ask. Your answer is handed back to you
next time as `the_goal_you_chose_last_time`: it is how a model that cannot
write anything keeps an intention. Hold a goal while it is still right rather
than changing it every turn.

descend = get to the next level down; press > only while on a down staircase
explore = uncover this level to find a down staircase or something worth taking
fight = kill what is in front of you, or finish a fight already started
recover = rest or heal until it is safe to carry on
equip = buy, pick up, wear or wield something the character needs
answer_the_prompt = a question, menu or -more- is on screen and comes first

## commands

The game's own name for each key is added automatically from
`docs/command.rst` at the game pin. What follows is what this manual adds:
when the key applies.

> = go down a staircase. Only while standing on one; the status line at the
    bottom of the screen says "Down staircase" when you are
< = go up a staircase. Only while standing on one, and going up is almost
    never what you want. After descending you arrive on an up staircase, so
    this is the key that undoes the progress you just made
. = run in a direction, which you give next. The game walks you along the
    corridor until something happens, far cheaper than stepping
R = rest; the game then asks how long, and & rests until healed or disturbed
i = list what you are carrying. Worth pressing after shopping and whenever
    you are unsure, because nothing here remembers it for you
e = list what you are wearing and wielding
q = quaff a potion, then its letter. Cure Light Wounds answers being hurt
r = read a scroll, then its letter. Phase Door answers being cornered
v = throw something, then its letter, then a target. This is how oil is used
w = wear or wield something, then its letter
g = pick up what is under you
p = in a store, buy. In the dungeon, walk to the nearest unknown square, but
    only if the autoexplore option is on, and it is off by default
s = in a store, sell
5 = stay still for a turn, or confirm the current target
y = yes, to a question ending in [y/n]
n = no, to a question ending in [y/n]
