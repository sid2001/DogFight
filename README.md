# A Space Shooter Game

The game presents a simple survival type play through

## Physics 
 - The player applies throttle which comes with three stage throttle with varying acceleration and dynamic drag to enhance flying experience. It applies basic newtonian physics.
 - The bots have their own throttle system which is dynamically adjusted based on their distance from the player and to avoid collision among themselves. The accelerated motion gives a realistic mechanism to bots making them vulnerable to collision if not controlled optimally.
 - The swarm bots implements three types of mechanism to create a flock
     * Cohesion: Bots try to move towards other bots in the close proximity
     * Alignment: Bots align with the swarm leader to follow in the same direction.
     * Adhesion: Bots try to avoid collision when they get too close with each other.
 - The enviroment replicates a simple solar system with the planets rotating on it's axis and around the Sun.
 - All entites are enclosed with collision meshes which is used to detect collision.
 - To detect collision, it uses oct-tree data structure to divide the 3d space spatially into octants to make the collision detection more efficient. O(nlog(n)) time from O(n<sup>2</sup>).
 - Bigger bots can avoid obstacles while chasing the player making the game more challenging.

## State Management and Resource Management
> I have tried to implement better resource management to avoid high memory usage through dynamic resource loading. It is not very efficient yet since being a learning project I have only implemented it for few assets and basic state management to go in and out of menu screen and states like game over.

## Weapon System
 - Turret: This weapon comes with infinite bullets but requires high precision to shoot down enemies
 - Homing Missile: With limited ammo this provides locking down on big enemy bots. These are not effective towards swarm bots due to their size.
 - Swarm Missile: An infinite ammo with cooldown period, this weapon is designed to chase and kill swarm bots.

## Sounds
A spatial sound system to give palyers a more imerssive experience
 - Engine humming
 - Turret sound
 - Missile launch and cruise sound
 - Explosions

## Demo

Oct-tree spatial division for better collision detection

### ![Untitled Project (1)](https://github.com/user-attachments/assets/44277456-6402-430a-a24c-ed396946a35b)

Swarm enemies can flock with their surrounding bots

### ![swarm-ezgif com-optimize](https://github.com/user-attachments/assets/f51d2186-2d05-4c4b-832f-48261fbdb49b)

With homing missiles you can lock on enemies and shoot at them. These missiles are not effective towards swarm bots due to their size


![homing](https://github.com/user-attachments/assets/e2009314-e400-426d-8606-6ba96e3e4cb7)


Swarm missiles are designed to track and kill swarm bots. 

![sarmmissile-ezgif com-optimize](https://github.com/user-attachments/assets/05c69d5c-a85e-42b7-abe1-bb1effaf15d4)

## Todo
 - HUD display
 - Multiplayer
 - Aim assist
 - Display score

## To play
```
git clone git@github.com:sid2001/DogFight.git

cd DogFight

cargo build --release && cargo run
