# Bevy game jam #6

Entry to the Bevy game jam #6 [https://itch.io/jam/bevy-jam-6](https://itch.io/jam/bevy-jam-6).

## Theme

Chain reaction

## Concept

Idle Sparks

Idle incremental game on a simple grid, 9x9 to start with.

Buy and place nodes on the grid that "pulsate" and generate "Spark points".

Some nodes can trigger other nodes.

## prio

- [x] Draw and navigate grid
- [x] Place a "clicker node" when game starts at (4,4)
  - [x] navigate to clicker node
  - [x] press either space,enter or i
  - [x] increment spark points
- [x] Place a "stimulator node"
  - [x] navigate to an empty cell
  - [x] press either space,enter,b, or i to enter build mode
  - [x] display overlay with buyable spark nodes
  - [x] press corresponding key to build spark node at position
  - [x] stimulator node should click neighboring clicker nodes
  - [x] cost 20

## bugs

- [ ] click observer Pointer<Released> triggers twice..?

## polish

- [x] Animate clicker and simulator nodes
- [x] UI info, navigate and key bindings etc
- [ ] animate numbers above a node when it generates spark points
- [x] add popping sounds or spark sounds from Kenny audios libs
- [x] animate clicker node when interacted with
- [ ] Animate crosshair/focus animation between cells

## stretch goals

- [ ] build clicker node as well

## refactor

- [ ] send build events to build stuff and deduct SP
- [ ] delay event task
