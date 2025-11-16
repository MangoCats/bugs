# CLAUDE.md - AI Assistant Guide for Bugs Repository

## Project Overview

**Bugs** is a genetic programming experiment simulating evolution of virtual organisms ("bugs") in a 2D hexagonal grid world. Originally created in 2003 by Mike Inman, this project demonstrates emergent behavior through genetic algorithms, natural selection, and environmental pressures.

### Key Concepts
- **Genetic Programming**: Bugs have evolving genomes that control behavior
- **Natural Selection**: Bugs compete for food, mate, and reproduce
- **Environmental Dynamics**: Seasonal food growth, population pressures, terrain features
- **Visualization**: Generates image sequences showing population evolution over time

## Repository Structure

```
bugs/
├── LICENSE              # MIT License (Copyright 2020 MangoCats)
├── 0.23/               # Version 0.23 (2244 lines)
│   └── bugs.c
├── 0.24/               # Version 0.24 (2268 lines)
│   └── bugs.c
├── 0.25/               # Version 0.25 (2276 lines)
│   └── bugs.c
├── 0.26/               # Version 0.26 (2397 lines)
│   └── bugs.c
├── 0.27/               # Version 0.27 (2413 lines)
│   └── bugs.c
├── 0.28/               # Version 0.28 (2483 lines)
│   └── bugs.c
└── 0.29/               # Version 0.29 (Latest, 2492 lines, 1920x1080 world)
    └── bugs.c
```

### Version Evolution
Each directory contains a complete standalone version of the simulation. Versions are sequential refinements adding features like:
- Dynamic population control
- Mating requirements (on/off cycles)
- Ethnicity tracking and visualization
- Terrain features (cosine-based zones)
- Food growth regulation
- World size increases (0.29: 1920x1080)
- Various cost/parameter tuning

## Building and Running

### Dependencies
```bash
sudo apt install libgd-dev  # GD Graphics Library for image generation
```

### Compilation
```bash
cd 0.29  # Or any version directory
gcc bugs.c -lgd -lm -o bugs
```

### Execution
```bash
./bugs
```

The program will:
1. Initialize a world with food distribution
2. Create initial bug(s) with seed genetics
3. Simulate evolution over many turns (seasons/years)
4. Generate image files (b*.jpg) showing population state
5. Output text reports with statistics

### Creating Videos from Output
```bash
# Using ffmpeg (recommended for 0.29)
ffmpeg -f image2 -pattern_type glob -framerate 60 -i 'b*.jpg' -s 1920x1080 bugs029.mp4

# Alternative using convert/mpeg2enc
convert b*.jpg ppm:- | ppmtoy4m -S 420_mpeg2 -v 2 | mpeg2enc -o bugs022.m1v -f 3 -v 2 -b 7500 -I 0 -n n -H -M 0
```

## Code Architecture

### Core Data Structures

#### World Model
```c
WORLD_X × WORLD_Y           // Grid dimensions (1760×1000 in v0.29)
struct _worlddata           // Each cell contains:
  - food[][]                // Food quantity (fixed-point * 1024)
  - bug[][]                 // Pointer to bug in cell
  - ethnicity[][]           // Color/ethnicity tracking
```

#### Bug Structure
```c
struct _bugdata {
  struct _bugbrain *brain;  // Genetic program
  struct _bugstate state;   // Position, facing, weight, hydration
  long birthday;            // Age tracking
  long kills, defends;      // Life history
  long mate_success, spawn_success;
}
```

#### Genetic Programming
```c
struct _gene {
  long tp;                  // Gene type: 1=constant, 2=sense, 3=limit, 4=compare, 5=match
  long si;                  // Sense index
  long c1, c2;              // Constants
  struct _gene *next, *prev; // Linked list
  struct _gene *prod, *sum;  // Expression tree (value = vself * vprod + vsum)
}

struct _bugbrain {
  struct _bugact act[8];    // One chromosome pair per decision type
  long generation;          // Ancestry depth
  long divide;              // Reproduction strategy (2-7 children)
  short ngenes;             // Total gene count (affects metabolism cost)
  short expression;         // Bitmap: which chromosomes are active
}
```

### Bug Actions (Decisions)
Bugs evaluate their genetic programs to weight these actions:
- `ACTSLEEP` (0): Rest, minimal energy cost
- `ACTEAT` (1): Consume food from current cell
- `ACTTURNCW` (2): Turn clockwise
- `ACTTURNCCW` (3): Turn counter-clockwise
- `ACTMOVE` (4): Move forward (or fight if cell occupied)
- `ACTMATE` (5): Mate with bug in front
- `ACTDIVIDE` (6): Reproduce asexually or with mate

### Sensing System
Bugs perceive their environment through ~79 senses (NSENSES):
- **12 cells in 3 directions** × 4 items = 72 senses:
  - `ITEMFOOD`: Food quantity in cell
  - `ITEMBUG`: Bug mass in cell
  - `ITEMBUGFACE`: Bug's facing direction
  - `ITEMBUGMATCH`: Genetic similarity (0-1.0)
- **Self senses**:
  - `SENSESELF`: Own characteristics
  - `SELFAGE`: Time since birth
  - `SPAWNWEIGHTNORM`: Readiness for reproduction
  - `STARVEWEIGHTNORM`: Starvation indicator
  - `THIRSTSENSE`: Hydration level

### Key Functions

#### Simulation Loop (main)
1. `init_world()` - Initialize terrain, food, first bug(s)
2. Per-turn loop:
   - `grow_food()` - Seasonal food growth with terrain variation
   - `flow_water()` - Hydration/drowning mechanics
   - `move_bugs()` - Execute bug actions
   - `update_nearest()` - Update proximity tracking
3. Periodic reporting and visualization

#### Bug Behavior
- `gather_senses()` - Collect environmental data
- `bug_move()` - Evaluate genetics, select action, execute
- `mutatebrain()` - Apply random mutations to offspring
- `tweakgene()` - Modify individual genes during mutation

#### Reproduction
- Offspring receive mutated genes from parent(s)
- Mating combines chromosomes from two bugs
- Division can produce 2-7 children (genetically determined)
- Energy cost proportional to offspring count

#### Evolution Pressures
- **Energy costs**: Movement, turning, eating, mating, thinking (gene count)
- **Starvation**: Below DIETHIN (100 units), bug dies
- **Population limit**: Dynamic target, hard cap enforced
- **Seasonal variation**: Food growth varies by location and time
- **Gene cost**: Complex genomes require more energy
- **Forced mating cycles**: Periodic requirements/allowances

## Development Conventions

### Code Style
- **K&R C style**: Compact, minimal whitespace
- **Fixed-point arithmetic**: Many values × 1024 for precision
- **Global state**: World data stored in global arrays
- **Extensive comments**: Each version includes detailed revision history

### Configuration Constants
Key parameters are `#define` constants at the top of each file:
```c
WORLD_X, WORLD_Y         // World dimensions
FOODGROW, FOODDECAY      // Food dynamics
COSTSLEEP, COSTEAT, etc. // Action energy costs
GENECOST, GENEKNEE       // Genetic complexity penalties
POP_TARGET               // Population control
```

### Versioning Strategy
- **New directory per version**: Each version is self-contained
- **Revision history in comments**: Comprehensive changelog at file top
- **Parameter tuning**: Most changes involve constant adjustments
- **Feature additions**: Gradual complexity increases

### Memory Management
- Bugs and genes use `malloc`/`free`
- Linked lists for dynamic gene structures
- Explicit cleanup in `free_genes()`, `free_brain()`, `kill_bug()`

## Working with This Codebase

### For AI Assistants

#### Analysis Tasks
When analyzing this code:
1. **Check version first**: Specify which version (0.23-0.29)
2. **Read revision history**: Lines 13-170 contain detailed changelog
3. **Understand fixed-point**: Many values are scaled × 1024
4. **Trace gene evaluation**: Follow `prod`/`sum` tree structure

#### Modification Tasks
When modifying:
1. **Create new version**: Copy latest version to new directory (e.g., 0.30/)
2. **Update revision history**: Add entry at top of changelog
3. **Test compilation**: `gcc bugs.c -lgd -lm -o bugs`
4. **Document changes**: Explain parameter adjustments and rationale

#### Common Modification Patterns
- **Tuning evolution**: Adjust `COST*` constants, `GENEKNEE`, population targets
- **World changes**: Modify `WORLD_X`/`WORLD_Y`, `grow_food()` terrain logic
- **New senses**: Add to sensing array, update `NSENSES`, modify `gather_senses()`
- **New actions**: Add action constant, update decision logic in `bug_move()`
- **Visualization**: Modify `image_plot()` for different rendering

### Important Notes

#### The "Comet Strike"
Version 0.18 includes a deliberate bug causing mass extinction around year 75 due to overflow in food growth calculations. This is documented and intentionally preserved as part of the experiment.

#### Hexagonal Grid
The world uses hexagonal tiling (6 neighbors):
```
Directions: E(0), NE(-1), NW(-2), W(3), SW(2), SE(1)
Movement functions: east(), northeast(), northwest(), west(), southwest(), southeast()
```

#### Ethnicity System
- Each bug has RGB ethnicity values
- Children inherit and blend parent colors
- Ethnicity assimilates to local population over `ETHNIC_DUR` generations
- Creates visually distinct population lineages

#### Performance Considerations
- Large worlds (1920×1080) generate significant data
- Image generation is CPU-intensive (libgd)
- Long simulations produce many output files
- Consider disk space for b*.jpg image sequences

## Git Workflow

### Current State
- Repository: `MangoCats/bugs`
- License: MIT (2020 MangoCats)
- Original author: Mike Inman (2003)
- Active branch: `claude/claude-md-mi15jeqb90hbogvq-01GEuz444XMJuXkzxEDbCtSW`

### Making Changes
1. Work on designated branch (starts with `claude/`)
2. Commit with descriptive messages explaining parameter changes
3. Push to origin with: `git push -u origin <branch-name>`
4. Create pull requests for significant features

### Commit Message Style
```
Add version 0.30 with enhanced terrain features

- Increase WORLD_Y to 1200 for better aspect ratio
- Add mountain/valley elevation to food growth
- Adjust FOODGROW to compensate for terrain variation
```

## Experimental Nature

This is a **research/educational project** exploring:
- Emergent complexity from simple rules
- Genetic algorithms and evolution
- Artificial life simulation
- Data visualization

When working with this code, embrace the experimental approach:
- Try parameter variations
- Observe emergent behaviors
- Document unexpected results
- Preserve historical versions

## Reference Resources

### Understanding Genetic Programming
- Bugs use tree-based gene expression (prod/sum links)
- Each decision has two chromosomes (diploid genetics)
- Expression bitmap controls which chromosomes evaluate
- Mutation adds/removes/modifies genes randomly

### Key Papers/Inspiration
Original inspiration: 1992 Scientific American article (referenced but unread by original author)

### Related Concepts
- Tierra (digital evolution platform)
- Avida (artificial life simulation)
- Core Wars (programming game)
- Conway's Game of Life (cellular automaton)

---

**Last Updated**: 2025-11-16
**Latest Version**: 0.29 (1920×1080 world)
**Maintained by**: MangoCats
**Original Author**: Mike Inman (2003)
