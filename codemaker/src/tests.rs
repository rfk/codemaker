/* Copyright 2021 Ryan F Kelly
 *
 * Licensed under the Apache License (Version 2.0), or the MIT license,
 * (the "Licenses") at your option. You may not use this file except in
 * compliance with one of the Licenses. You may obtain copies of the
 * Licenses at:
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *    http://opensource.org/licenses/MIT
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the Licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the Licenses for the specific language governing permissions and
 * limitations under the Licenses. */
use super::*;

#[test]
fn test_define_rules_on_static_target() {
    struct TestMaker;

    define_codemaker_rules! {
        TestMaker as self {
            Vec<u32> as input => String {
                self.make_from_iter(input.into_iter()).collect()
            }
            &Vec<u32> as input => String {
                self.make_from_iter(input.iter().cloned()).collect()
            }
            u32 as input => String {
                format!("{},", input)
            }
        }
    }

    let t = TestMaker {};
    assert_eq!(t.make_from(vec![1, 2, 3]), "1,2,3,");
    assert_eq!(t.make_from(&vec![4, 5, 6, 7]), "4,5,6,7,");
}

#[test]
fn test_define_rules_on_target_with_anonymous_lifetime_on_target() {
    struct TestConfig {
        sep: String,
    }
    struct TestMaker<'a> {
        config: &'a TestConfig,
    }

    define_codemaker_rules! {
        TestMaker<'_> as self {
            Vec<u32> as input => String {
                self.make_from_iter(input).collect()
            }
            &Vec<u32> as input => String {
                self.make_from_iter(input.iter()).collect()
            }
            u32 as input => String {
                format!("{}{}", input, self.config.sep)
            }
            & u32 as input => String {
                format!("{}{}", input, self.config.sep)
            }
        }
    }

    let c = TestConfig { sep: "-".into() };
    let t = TestMaker { config: &c };
    assert_eq!(t.make_from(vec![1, 2, 3]), "1-2-3-");
    assert_eq!(t.make_from(&vec![4, 5, 6, 7]), "4-5-6-7-");
}
