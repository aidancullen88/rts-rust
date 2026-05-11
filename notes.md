Idea is:

Npcs struct contains the npcs map and the cell map, contains everything for adding new, selecting, getting
lists/specific npcs maybe: functions like decide_tasks, update_movement etc exist within npcs


For cover movement: they can't just stop moving as they will overlap other npcs, they should go into another movement
routine to move them away
