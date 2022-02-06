# PV251 Project -- Visualization of Parametrized Boolean Networks (PBNs)

_Boolean networks_ (BN) provide a valuable framework for modeling bio-logical
processes. By introducing _parameters_ into the model, various possible
behaviors can be considered and analyzed.

More concretely, a BN consists of interconnected _boolean variables_.
Each variable describes a presence/absense of one component in the studied
system. Interconnections describe their mutual influence. A PBN adds
parameters into the model -- it means, more possible influences. We convert
a BN to a _state transition graph_ (STG), where each node relates to one
assignment to all of the boolean variables and there is a directed edge
between states A and B if the system can move from assignment A to B by
modeled influences. Adding parameters then leads to _colored_ STG (CSTG),
where each edge has a color corresponding to one concrete parametrization.
So we have multiple simple STGs describing various possible state transitions.

This tool can open a PBN in the .aeon format (You can find some in
[this repository](https://github.com/sybila/biodivine-boolean-models) and modify
or create new using [Aeon](http://biodivine.fi.muni.cz/aeon/)). Then it let
the user choose one of the possible colors/parametrizations of loaded PBN.
Unfortunately, working with all parametrisations at once seems to be harder
than we had thought in the beginning (primarily visualization). The explicit
BN is transformed to a STG and condensed to smaller directed acyclic graph
with _strongly connected components_ (SCC) representing some (possibly)
long-term meta-state, where the living system may stay for a while, moving
just through the states in this SCC. This condensed STG is then visualized.

## Running the tool

Tu run the tool, You'll need a
[Rust compiler](https://www.rust-lang.org/tools/install) and Cargo.
[Steps](https://doc.rust-lang.org/cargo/getting-started/installation.html).
To download all the required dependencies (Rust crates) and build the tool,
run `cargo build` in the root directory. `cargo run model.aeon` starts the
application, change `model.aeon` to a path to Your desired model. To quickly
try the tool, look in the `data/` directory.

You can move in the 3D visualization window using Your mouse. Left button for
rotation, right button for translation and scroll for zooming. The bigger
the node, the larger is the corresponding SCC. Left click on a node prints
all the states in this node and their number. Red nodes are roots and
green nodes are leaves.

## Limitations
Well, there is a lot of them ... with the most disappointing one for us is that,
after all, this is not a _landscape_. In larger models, this kind of
visualization gets very messy. Visualizing a landscape of states and
state transitions is a good idea, abstracting away from some details
in the model (that are the cause of messy graphs now). But it needs more
research on how to do that. How to condense nodes in another way, not only
to SCCs, because this is too strong condition? Some STGs have all the SCCs
trivial (consisting of only one state). How to map nodes to points on the
landscape? May be we do not get a planar condensed STG and how to resolve
edge crossing? We have tried to redce the number of edges by working only
with a transitive reduction of the STG, but it is still too dense.

## Future work?
Perhaps. First steps will be getting more into computer graphics and
algorithmic graph theory.
