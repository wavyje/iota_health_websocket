use sha2::{Digest, Sha256};

pub fn generate_merkle_tree(firstName: String, lastName: String, birthday: String, birthplace: String, nationality: String, address: String, hashedImage: String, expire: String) -> String {
    let mut data = [firstName, lastName, birthday, birthplace, nationality, address, hashedImage, expire];

    for element in data.iter_mut() {
        let mut hasher = Sha256::new();
        hasher.update(&element);
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);

        *element= result_format;
    }

    //leafA consists of hashedImage and firstName
    let leaf_a = {
        let mut hasher = Sha256::new();
        hasher.update(String::from(&data[6]) + &String::from(&data[0]));
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    //leafB consists of lastName and birthplace
    let leaf_b = {
        let mut hasher = Sha256::new();
        hasher.update(String::from(&data[1]) + &String::from(&data[3]));
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    //leafC consists of expire and birthday
    let leaf_c = {
        let mut hasher = Sha256::new();
        hasher.update(String::from(&data[7]) + &String::from(&data[2]));
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    //leafD consists of address and nationality
    let leaf_d = {
        let mut hasher = Sha256::new();
        hasher.update(String::from(&data[5]) + &String::from(&data[4]));
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    
    //calculate leafAB and leafCD
    let leaf_ab = {
        let mut hasher = Sha256::new();
        hasher.update(leaf_a + &leaf_b);
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    let leaf_cd = {
        let mut hasher = Sha256::new();
        hasher.update(leaf_c + &leaf_d);
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };
    println!("AB {}", &leaf_ab);
    println!("CD {}", &leaf_cd);
    //calculate root hash
    let root = {
        let mut hasher = Sha256::new();
        hasher.update(leaf_ab + &leaf_cd);
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);
        result_format
    };

    
    
    println!("root {}", &root);
    return root;
}