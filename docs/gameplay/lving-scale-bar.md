# 🎰 Living Scale Bar UI

A dynamic UI element that provides an intuitive, non-numeric sense of the player's current zoom level.

## Core Concept

As the camera zooms in and out, a standard scale bar (e.g., "1m," "10m") is paired with a visual "wheel" of animal icons, similar to a scrolling fruit machine.

## Key Components

- **Scale Text**: A label that displays the current scale in numeric form (e.g., "1cm," "10m," "1km").
- **Icon Wheel**: A vertical list of animal icons, each tied to a specific real-world size.
- **Mask/Window**: A UI element that "frames" the icon wheel, so only one or two animals are clearly visible at a time.

## Dynamic Behavior

- When the camera's zoom level changes, the system calculates the current world scale.
- It looks up this scale in a predefined, sorted data structure (e.g., `[ {scale: 0.1, icon: 'shrew'}, {scale: 1.0, icon: 'capybara'} ... ]`) to find the "closest match" animal.
- The icon wheel animates its scroll position, spinning to center the new target animal's icon within the mask.
- This animation should "snap" or "ease-out" to create a tactile, mechanical "fruit machine" feel.

# 🐾 Cool Animal Scale List

Here is a list of interesting animals to use for the wheel, progressing from tiny to huge.

## Tiny (cm-Scale)

- Tardigrade (Water Bear)
- Hummingbird
- Pygmy Shrew

## Small (sub-1m)

- Fennec Fox
- Platypus
- Hyrax
- Kiwi

## Medium (1m-2m)

- Capybara
- Thylacine (Tasmanian Tiger)
- Giant Anteater
- Komodo Dragon

## Large (2m-5m)

- Okapi
- Tapir
- Moose
- Elephant Seal

## Massive (5m+)

- Giraffe
- Indricotherium (Largest land mammal)
- Argentinosaurus (Dinosaur)
- Blue Whale