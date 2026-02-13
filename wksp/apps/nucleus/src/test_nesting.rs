pub fn test_nesting() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            println!("Deeply nested!");
                        }
                    }
                }
            }
        }
    }
}