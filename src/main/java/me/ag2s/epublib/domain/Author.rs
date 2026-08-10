// package me.ag2s.epublib.domain;

// import me.ag2s.epublib.util.StringUtil;

// import java.io.Serializable;

/**
 * Represents one of the authors of the book
 *
 * @author paul
 */
pub struct Author {
    firstname: String,
    lastname: String,
    relator: Relator,
}

impl Author {

    pub fn new(single_name: String) -> Author {
        Author::with_names("".to_string(), single_name)
    }

    pub fn with_names(firstname: String, lastname: String) -> Author {
        Author {
            firstname: firstname,
            lastname: lastname,
            relator: Relator::AUTHOR,
        }
    }

    pub fn get_firstname(&self) -> &String {
        &self.firstname
    }

    pub fn set_firstname(&mut self, firstname: String) {
        self.firstname = firstname;
    }

    pub fn get_lastname(&self) -> &String {
        &self.lastname
    }

    pub fn set_lastname(&mut self, lastname: String) {
        self.lastname = lastname;
    }


    // @Override
    // @SuppressWarnings("NullableProblems")
    pub fn to_string(&self) -> String {
        format!("{}, {}", self.lastname, self.firstname)
    }

    pub fn hash_code(&self) -> i32 {
        StringUtil::hash_code(&self.firstname, &self.lastname)
    }

    pub fn equals(&self, author_object: &dyn Any) -> bool {
        if !(author_object.is::<Author>()) {
            return false;
        }
        let other = author_object.downcast_ref::<Author>().unwrap();
        return StringUtil::equals(&self.firstname, &other.firstname)
                && StringUtil::equals(&self.lastname, &other.lastname);
    }

    /**
     * 设置贡献者的角色
     *
     * @param code 角色编号
     */

    pub fn set_role(&mut self, code: String) {
        let mut result = Relator::by_code(&code);
        if result.is_none() {
            result = Some(Relator::AUTHOR);
        }
        self.relator = result.unwrap();
    }

    pub fn get_relator(&self) -> Relator {
        self.relator
    }


    pub fn set_relator(&mut self, relator: Relator) {
        self.relator = relator;
    }
}
