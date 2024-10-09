#[cfg(test)]
mod tests {
    #[test]
    fn test_sand_gravity() {
        // Arrange
        let mut world = new_world();

        let sand_type = new_sand_block_type(&mut world);
        let sand_position = new_position(0, 0, 0);
        let sand = new_block(&mut world, &sand_type, &sand_position);

        let air_type = new_air_block_type(&mut world);
        let air_position = new_position(0, 1, 0);
        let air = new_block(&mut world, &air_type, &air_position);

        // Act
        tick_until_sand_falls_one_block(&mut world);

        // Assert
        assert_eq!(is_block_type_at_position(&world, &sand_position, &air_type), true);
        assert_eq!(is_block_type_at_position(&world, &air_position, &sand_type), true);
    }

    #[test]
    fn test_chest_inventory_putting_getting() {
        // Arrange
        let mut world = new_world();
        let chest_type = new_chest_block_type(&mut world);
        let mut chest = new_block(&mut world, &chest_type, &new_position(0, 0, 0));

        let pickaxe_type = new_pickaxe_equipment_type();
        let pickaxe = new_equipment(pickaxe_type);

        // Act
        add_equipment_to_chest(&mut chest, &pickaxe);

        // Assert
        assert_eq!(chest_contains_equipment(&chest, &pickaxe), true);
    }

    #[test]
    fn test_trap_door_opening_closing() {
        // Arrange
        let mut world = new_world();
        let trap_door_type = new_trap_door_block_type(&mut world);
        let trap_door = new_block(&mut world, &trap_door_type, &new_position(0, 0, 0));

        // Assert
        assert_eq!(is_trap_door_open(&trap_door), false);

        // Act
        use_trap_door(&mut world, &trap_door);

        // Assert
        assert_eq!(is_trap_door_open(&trap_door), true);

        // Act
        use_trap_door(&mut world, &trap_door);

        // Assert
        assert_eq!(is_trap_door_open(&trap_door), false);
    }

    #[test]
    fn test_text_sign() {
        // Arrange
        let mut world = new_world();
        let text_sign_type = new_text_sign_block_type(&mut world);
        let mut text_sign = new_block(&mut world, &text_sign_type, &new_position(0, 0, 0));
        let text = "Hello, World!".to_string();

        // Act
        set_text_of_text_sign(&mut text_sign, text);

        // Assert
        assert_eq!(get_text_of_text_sign(&text_sign), text);
    }

    #[test]
    fn test_water_spreading() {
        // Arrange
        let mut world = new_world();
        let water_type = new_water_block_type(&mut world);
        let water_position = new_position(0, 0, 0);
        let water = new_block(&mut world, &water_type, &water_position);

        let air_type = new_air_block_type(&mut world);
        let air_position = new_position(0, 1, 0);
        let air = new_block(&mut world, &air_type, &air_position);

        // Act
        tick_until_water_spreads_one_block(&mut world);

        // Assert
        assert_eq!(is_block_type_at_position(&world, &water_position, &air_type), true);
        assert_eq!(is_block_type_at_position(&world, &air_position, &water_type), true);

        // TODO: check if water is spreading in all directions
    }

    #[test]
    fn test_piston_move_neighbour_block() {
        // Arrange
        let mut world = new_world();
        let piston_type = new_piston_block_type(&mut world);
        let piston_position = new_position(0, 0, 0);
        let piston = new_block(&mut world, &piston_type, &piston_position);

        let sand_type = new_sand_block_type(&mut world);
        let sand_position = new_position(0, 1, 0);
        let sand = new_block(&mut world, &sand_type, &sand_position);

        // Act
        tick_until_piston_moves_one_block(&mut world);

        // Assert
        // TODO: check if piston moved sand
    }

    #[test]
    fn test_brick_equipment_requirements() {
        // Arrange
        let mut world = new_world();
        let brick_type = new_brick_block_type(&mut world);
        let brick_position = new_position(0, 0, 0);
        let brick = new_block(&mut world, &brick_type, &brick_position);

        let pickaxe_type = new_pickaxe_equipment_type();
        let pickaxe = new_equipment(pickaxe_type);

        let mut player = new_player();

        equip_player(&mut player, &pickaxe);

        // Assert
        assert_eq!(is_block_type_at_position(&world, &brick_position, &brick_type), true);

        // Act
        use_player_equipment_on_block_at_position(&mut player, &mut world, &brick_position);

        // Assert
        // TODO: check if is block type at position is air
        assert_eq!(is_block_type_at_position(&world, &brick_position, &brick_type), false);
    }

    #[test]
    fn test_barrier_unbreakable() {
        // Arrange
        let mut world = new_world();

        let barrier_type = new_barrier_block_type(&mut world);
        let barrier_position = new_position(0, 0, 0);
        let barrier = new_block(&mut world, &barrier_type, &barrier_position);

        let pickaxe_type = new_pickaxe_equipment_type();
        let pickaxe = new_equipment(pickaxe_type);

        let mut player = new_player();

        equip_player(&mut player, &pickaxe);

        // Assert
        assert_eq!(is_block_type_at_position(&world, &barrier_position, &barrier_type), true);

        // Act
        use_player_equipment_on_block_at_position(&mut player, &mut world, &barrier_position);

        // Assert
        assert_eq!(is_block_type_at_position(&world, &barrier_position, &barrier_type), true);
    }

    #[test]
    fn test_wire_using_block_in_front_when_used() {
        // Arrange
        let mut world = new_world();
        let wire_type = new_wire_block_type(&mut world);
        let wire = new_block(&mut world, &wire_type, &new_position(0, 0, 0));

        let brick_type = new_brick_block_type(&mut world);
        let brick = new_block(&mut world, &brick_type, &new_position(0, 1, 0));

        // Act
        use_wire(&mut world, &wire);

        // Assert
        // TODO: check if brick is used
    }


    #[test]
    fn test_clock_using_block_in_front_periodically() {
        // Arrange
        let mut world = new_world();
        let clock_type = new_clock_block_type(&mut world);
        let clock = new_block(&mut world, &clock_type, &new_position(0, 0, 0));

        let brick_type = new_brick_block_type(&mut world);
        let brick = new_block(&mut world, &brick_type, &new_position(0, 1, 0));

        // Act
        wait_until_clock_ticks(&mut world);

        // Assert
        // TODO: check if brick is used
    }

    #[test]
    fn test_player_moves() {
        // Arrange
        let mut world = new_world();
        let player = new_player(&mut world, &new_position(0, 0, 0));

        // Assert
        assert_eq!(is_player_position(&mut world, &player, &new_position(0, 0, 0)), true);

        // Act
        move_player(&mut world, &player, &new_position(0, 1, 0));

        // Assert
        assert_eq!(is_player_position(&mut world, &player, &new_position(0, 1, 0)), true);
    }

    #[test]
    fn test_die_block_kills_player() {
        // Arrange
        let mut world = new_world();

        let die_block_type = new_die_block_type(&mut world);
        let die_block_position = new_position(0, 0, 0);
        let die_block = new_block(&mut world, &die_block_type, &die_block_position);

        let player = new_player(&mut world, &new_position(0, 1, 0));

        // Assert
        assert_eq!(is_player_alive(&player), false);

        // Act
        move_player(&mut world, &player, &die_block_position);

        // Assert
        assert_eq!(is_player_alive(&player), false);
    }

    #[test]
    fn test_checkpoint_block_saves_history() {
        // TODO
    }

    #[test]
    fn test_checkpoint_block_restores_history() {
        // TODO
    }

    #[test]
    fn test_player_places_block() {
        // Arrange
        let mut world = new_world();
        let mut player = Player::new();
        let mut block = Block::new(BlockType::Block);
        let mut player_position = Position::new(0, 0);
        let mut block_position = Position::new(0, 1);

        // TODO: placing player
        // TODO: player places block

        // Act
        world.update();

        // Assert
        assert_eq!(world.get_block(block_position), block);
    }

    #[test]
    fn test_player_uses_block() {
        // Arrange
        let mut world = new_world();
        let mut player = Player::new();
        let mut block = Block::new(BlockType::Block);
        let mut player_position = Position::new(0, 0);
        let mut block_position = Position::new(0, 1);

        world.add_block(block_position, block);

        // TODO: placing player
        // TODO: player uses block

        // Act
        world.update();

        // Assert
        // TODO
    }

    #[test]
    fn test_player_gets_equipment() {
        // Arrange
        let mut world = new_world();
        let mut player = Player::new();
        let mut equipment = Equipment::new(EquipmentType::Pickaxe);
        let mut equipment_position = Position::new(0, 0);

        world.add_item(equipment_position, equipment);

        // TODO: placing player
        // TODO: player gets equipment

        // Act
        world.update();

        // Assert
        // TODO: check player equipment
        assert_eq!(world.get_item(equipment_position), None);
    }

    #[test]
    fn test_player_puts_equipment() {
        // Arrange
        let mut world = new_world();
        let mut player = Player::new();
        let mut equipment = Equipment::new(EquipmentType::Pickaxe);
        let mut equipment_position = Position::new(0, 0);

        // TODO: placing player
        player.equip(equipment);
        // TODO: player puts equipment

        // Act
        world.update();

        // Assert
        // TODO: check player equipment
        assert_eq!(world.get_item(equipment_position), equipment);
    }

    struct Block {}
    struct BlockType {}
    struct Equipment{}
    struct EquipmentType{}
    struct Player{}
    struct Position {}
    struct World {}

    fn add_equipment_to_chest(chest: &mut Block, item: &Equipment) {
        todo!()
    }

    fn equip_player(player: &mut Player, equipment: &Equipment) {
        todo!()
    }

    fn new_air_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_barrier_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_block(world: &mut World, block_type: &BlockType, position: &Position) -> Block {
        todo!()
    }

    fn new_brick_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_clock_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_chest_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_die_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_equipment(equipment_type: EquipmentType) -> Equipment {
        todo!()
    }

    fn new_pickaxe_equipment_type() -> EquipmentType {
        todo!()
    }

    fn new_piston_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_player(world: &mut World, position: &Position) -> Player {
        todo!()
    }

    fn new_position(x: i32, y: i32, z: i32) -> Position {
        todo!()
    }

    fn new_sand_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_text_sign_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_trap_door_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_water_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_wire_block_type(world: &mut World) -> BlockType {
        todo!()
    }

    fn new_world() -> World {
        todo!()
    }

    // TODO: Should return not only Equipment but also Blocks?
    fn chest_contains_equipment(chest: &Block, equipment: &Equipment) -> bool {
        todo!()
    }

    fn is_block_type_at_position(world: &World, position: &Position, block_type: &BlockType) -> bool {
        todo!()
    }

    fn is_trap_door_open(trap_door: &Block) -> bool {
        todo!()
    }

    fn is_player_alive(player: &Player) -> bool {
        todo!()
    }

    fn is_player_position(world: &World, player: &Player, position: &Position) -> bool {
        todo!()
    }

    fn move_player(world: &mut World, player: &Player, position: &Position) {
        todo!()
    }

    fn get_text_of_text_sign(text_sign: &Block) -> String {
        todo!()
    }

    fn set_text_of_text_sign(text_sign: &mut Block, text: String) {
        todo!()
    }

    fn tick_until_sand_falls_one_block(world: &mut World) {
        todo!()
    }

    fn tick_until_water_spreads_one_block(world: &mut World) {
        todo!()
    }

    fn tick_until_piston_moves_one_block(world: &mut World) {
        todo!()
    }

    fn use_trap_door(world: &mut World, trap_door: &Block) {
        todo!()
    }

    fn use_wire(world: &mut World, wire: &Block) {
        todo!()
    }

    fn use_player_equipment_on_block_at_position(player: &mut Player, world: &mut World, position: &Position) {
        todo!()
    }

    fn wait_until_clock_ticks(world: &mut World) {
        todo!()
    }
}