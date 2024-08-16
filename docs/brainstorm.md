# Brainstorm 

The theme is **BUILT TO SCALE**

- scalable server infrastructure, distributing computing
- repeatable (tile-able) design (think factorio)
- zach-like production game (opus magnum, infinifactory, etc.)
- lizards and fish have scales
- buildings, walls
- robots are build, ironman (suits), engines
- scale can mean climb
- robot who climbs (Grow Home)
- scale can mean units (metres, celsius)
- scale can measure weight (of people, or other)
- scales of justice (judgement, law)
- dry scaly skin
- building something to real size (building to scale as opposed to miniature)
- musical scale (e.g. Eb minor pentatonic)
- "built" slang for strong, muscly, hard
- built = manufactured
- scale as stretching
- zooming in or out to change scale
    - CGP Grey Metric Paper: https://youtu.be/pUF5esTscZI?si=UlzqusKnLJZ_yUZo
    - carykh https://htwins.net/scale2/
- cells (many cells form tissue, which forms organs, etc.)
- many of a thing forms a new thing - at a larger scale
- mould, bacteria, bees (honeycombs)
- dystopia housing, overcrowding, tiny houses stacked on top, underground
- creative building (minecraft/roblox)
- panelaky (communist buildings)
- crane dropping things (Tower Bloxx)
- garbage collection / warehouse, have to build machines to recycle / store / distribute (Tetris, but things coming from all sides?)
- resource management
- horizontal and vertical scaling
- hive (bees), attract beekeeper with your hive, build larger hives, collect (Cookie Clicker??)
- scale =? scope
- tesselation

- ship engines
- trees can grow
- growing = changing scale
- mice that live in the tree, house becomes bigger as tree grows
- factorio-like, make components that work in a fractal way
- components that you can use as blocks
- finitely tile-able
- climbing paths, (puzzle) platformer
- climbing paths different for creatures of different sizes
- mech - step in and become bigger
- recursive mechs (matrioska dolls)
- climb the mech to get in the seat
- linear algebra (scales)
- affine spaces

- einstein tile (non-repeating tile)
- turn based platformer
- moving on a grid (chess pieces, onitama)

- Katamari
- Screw Drivers
- Cosmoteer
- Space Engineer
- Rodina
- StarMade

- first day on the job at a space station
- fixing engines
- building from a blueprint (small scale)
- pipe connection game / railtrack game

- Patricks Parabox
- Bomba
- game inside a game inside a game
- Portals, change your size by going through a portal

- Building sub-engines inside your engine
- Fractal engines (Star Trek)

- Build a mech to make a larger mech (recursively)
- AI for board game, card game

- TAS
- repeating sequence, see how far you can get
- fractal levels
- strategy game, (AI for it?)

- game of life / cellular automata
- momentum preserved
- destroying blocks, placing blocks in front
- physics based movement (Explodey)

- record all attempts and play them back once finishing game

- platformer
- sokoban
- shooter
- strategy game
- spreading / territory (like bacteria / cells)

- pathfinding by spreading chemicals

- landmarks (flag, sand, colour, pheromone) -> has an attached program, reaching it jumps to it

- baba is you

- Gridentify, 2048

- past decisions stack, and make things harder in the future
- arcade, high-score
- build a tower out of tetris pieces, realistic physics, high as possible
- bullet-hell

- build an engine for a ship that build larger engines
- Von Neumman probes (https://en.wikipedia.org/wiki/Self-replicating_spacecraft)
- Core War https://en.wikipedia.org/wiki/Core_War

- Reigns (make decisions which impact later ones)
- Spore

- City planning (growing city)
- Tower defense, where you cannot remove your towers
- maybe you need to power your towers, but you have to work around what you already built

- Build something, and then later you have to build on top of it

- Server infrastructure idea expanded: Packets come in, and you have to divide them (horizontal scaling), and get them to certain services, etc. (Graph building)
- Balance two sides of a scale; blocks of increasing sizes come in, physics based, build larger and larger piles


## Server Infrastructure

- More and more packets, different colors
- Servers need to be cooled
- Servers need to fit

- Graph / Network
- Incoming packets along one edge
- Different types of nodes: load balancer (allow multiple outgoing connections), cache, (micro)services, routers
- Packets of different color need different services, latency requirements
- Buffers, processing time for a packet

- Soft/hard deadlines, latency requirements
- Vertical scaling by adding more CPU / memory
- Replace broken hard drive
- Some applications need more database (memory), some need more compute
- Internal network

- Inspirations: mini metro, mini motorways

## Spaceship Engine

- Place engine pieces on a grid
- Connect pipes / wires
- Manage resources (Energy, antimatter, idk)
- Tile-able designs

- Size limit
- ~~Recursive spaceship building~~

## Repeating Instructions Platformer

- Write a cycle of instructions that repeats
- Complete all levels with the same cycle
- You can place flags which you can use for conditional code
- Conditional movement
- Place/remove blocks (construct paths)
- Maybe physics based movement (cool?)

- Movement options:
    - explodee rocket propulsion
    - simple step
    - directional jump
    - move blocks
    - trampoline
    - portals
    - ice
    - bigger and bigger momementum/levels
    - optional moves
    - ladder
    - grappling hook

- You have to reach a checkpoint to unlock an instruction
