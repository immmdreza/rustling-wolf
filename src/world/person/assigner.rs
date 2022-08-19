use super::roles::Role;

pub fn roles(len: u64) -> Vec<Role> {
    let mut arr = Vec::<Role>::new();
    for _ in 0..len {
        arr.push(Role::Villager);
    }

    match len {
        5 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
        }
        6 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
            arr[2] = Role::Doctor;
        }
        7 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
            arr[2] = Role::Doctor;
            arr[3] = Role::MasterWolf;
        }
        8 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
            arr[2] = Role::Doctor;
            arr[3] = Role::MasterWolf;
            arr[4] = Role::Villager;
        }
        9 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
            arr[2] = Role::Doctor;
            arr[3] = Role::MasterWolf;
            arr[4] = Role::Villager;
            arr[5] = Role::Villager;
        }
        10 => {
            arr[0] = Role::Wolf;
            arr[1] = Role::Seer;
            arr[2] = Role::Doctor;
            arr[3] = Role::MasterWolf;
            arr[4] = Role::Villager;
            arr[5] = Role::Villager;
            arr[6] = Role::Wolf;
        }
        _ => panic!("Invalid number of players"),
    };

    arr
}
