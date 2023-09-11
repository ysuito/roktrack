//! Provide Object Detection
//!
pub mod onnx {
    use crate::module::{define, util::init::RoktrackProperty};
    use image::{imageops::FilterType, io::Reader, ImageBuffer, Pixel, Rgb};
    use ndarray::{s, Array, Axis, IxDyn};
    use ort::{
        environment::Environment, value::Value, ExecutionProvider, GraphOptimizationLevel,
        LoggingLevel, Session, SessionBuilder,
    };
    use std::path::Path;

    use super::Detection;

    /// Session Types
    ///
    #[derive(Debug, Clone, PartialEq)]
    pub enum SessionType {
        Sz320, // basic 320 * 320 inference
        Sz640, // basic 640 * 640 inference
        Ocr,   // ocr 96 * 96 inference
    }
    /// Session Type methods
    ///
    impl SessionType {
        fn get_imgsz(&self) -> u32 {
            match self {
                Self::Sz320 => 320,
                Self::Sz640 => 640,
                Self::Ocr => 96,
            }
        }
    }
    /// Bundled Sessions
    ///
    pub enum Sessions {
        Pylon {
            sz320: Session,
            sz640: Session,
        },
        PylonOcr {
            sz320: Session,
            sz640: Session,
            ocr: Session,
        },
        Animal {
            sz320: Session,
            sz640: Session,
        },
    }

    /// YoloV8 session store.
    ///
    pub struct YoloV8 {
        pub sessions: Sessions,
        pub session_type: SessionType,
    }

    impl Default for YoloV8 {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Methods for yolov8.
    ///
    impl YoloV8 {
        /// yolov8's constructor.
        ///
        pub fn new() -> Self {
            Self {
                sessions: Self::build_pylon_sessions().expect("Can't initialize pylon sessions"),
                session_type: SessionType::Sz320,
            }
        }
        /// get session
        ///
        pub fn get_session(
            name: &str,
            model_path: &str,
        ) -> Result<Session, Box<dyn std::error::Error>> {
            let environment = Environment::builder()
                .with_name(name)
                .with_log_level(LoggingLevel::Warning)
                .with_execution_providers([ExecutionProvider::CPU(Default::default())])
                .build()?
                .into_arc();
            let session = SessionBuilder::new(&environment)?
                .with_optimization_level(GraphOptimizationLevel::Level1)?
                .with_intra_threads(8)?
                .with_model_from_file(model_path)?;
            Ok(session)
        }
        /// Build Pylon Session Bundle
        ///
        pub fn build_pylon_sessions() -> Result<Sessions, Box<dyn std::error::Error>> {
            let sessions = Sessions::Pylon {
                sz320: Self::get_session("pylon_sz320", define::path::PYLON_320_MODEL)?,
                sz640: Self::get_session("pylon_sz640", define::path::PYLON_640_MODEL)?,
            };
            Ok(sessions)
        }
        /// Build Pylon OCR Session Bundle
        ///
        pub fn build_pylon_ocr_sessions() -> Result<Sessions, Box<dyn std::error::Error>> {
            let sessions = Sessions::PylonOcr {
                sz320: Self::get_session("pylon_sz320", define::path::PYLON_320_MODEL)?,
                sz640: Self::get_session("pylon_sz640", define::path::PYLON_640_MODEL)?,
                ocr: Self::get_session("pylon_ocr", define::path::DIGIT_OCR_96_MODEL)?,
            };
            Ok(sessions)
        }
        /// Build Animal Session Bundle
        ///
        pub fn build_animal_sessions() -> Result<Sessions, Box<dyn std::error::Error>> {
            let sessions = Sessions::Animal {
                sz320: Self::get_session("animal_sz320", define::path::ANIMAL_320_MODEL)?,
                sz640: Self::get_session("animal_sz640", define::path::ANIMAL_640_MODEL)?,
            };
            Ok(sessions)
        }
        /// Infer
        ///
        pub fn infer(
            &self,
            impath: &str,
            session_type: SessionType,
        ) -> Result<Vec<super::Detection>, Box<dyn std::error::Error>> {
            let sz = session_type.get_imgsz();
            // Load image and resize to model's shape, converting to RGB format
            let img: ImageBuffer<Rgb<u8>, Vec<u8>> = image::open(Path::new(impath))?
                .resize_exact(sz, sz, FilterType::Nearest)
                .to_rgb8();

            let array = ndarray::CowArray::from(
                ndarray::Array::from_shape_fn((1, 3, sz as usize, sz as usize), |(_, c, j, i)| {
                    let pixel = img.get_pixel(i as u32, j as u32);
                    let channels = pixel.channels();
                    // normalize
                    // range [0, 255] -> range [0, 1]
                    (channels[c] as f32) / 255.0
                })
                .into_dyn(),
            );

            let session = match &self.sessions {
                Sessions::Pylon { sz320, sz640 } => match session_type {
                    SessionType::Sz320 => sz320,
                    SessionType::Sz640 => sz640,
                    _ => panic!("Invalid Session Type"),
                },
                Sessions::PylonOcr { sz320, sz640, ocr } => match session_type {
                    SessionType::Sz320 => sz320,
                    SessionType::Sz640 => sz640,
                    SessionType::Ocr => ocr,
                },
                Sessions::Animal { sz320, sz640 } => match session_type {
                    SessionType::Sz320 => sz320,
                    SessionType::Sz640 => sz640,
                    _ => panic!("Invalid Session Type"),
                },
            };

            let tensor = vec![Value::from_array(session.allocator(), &array)?];

            let outs = session.run(tensor)?;
            let out = outs
                .get(0)
                .unwrap()
                .try_extract::<f32>()?
                .view()
                .t()
                .into_owned();
            convert_yolo_fmt(out)
        }

        /// Whether the current session supports OCR
        pub fn support_ocr(&self) -> bool {
            match self.sessions {
                Sessions::Pylon { .. } => false,
                Sessions::PylonOcr { .. } => true,
                Sessions::Animal { .. } => false,
            }
        }

        /// Detects numbers in the vicinity of the marker.
        ///
        /// The bbox of the marker detected in low resolution is extracted
        /// from the high resolution image by ratio and applied to OCR.
        pub fn ocr(
            &self,
            impath: &str,
            dets: Vec<Detection>,
            property: RoktrackProperty,
        ) -> Result<Vec<Detection>, Box<dyn std::error::Error>> {
            // For result
            let mut new_dets = dets.clone();

            // Resolution of the image used for detection
            let (iw, ih) = match self.session_type {
                SessionType::Sz320 => (320.0, 320.0),
                SessionType::Sz640 => (640.0, 640.0),
                _ => (0.0, 0.0),
            };
            // Original resolution
            let (ow, oh) = (
                property.conf.camera.width as f64,
                property.conf.camera.height as f64,
            );
            // Load original image (full resolution)
            let mut img = Reader::open(impath)?.decode()?;
            // Iterates dets.
            for (i, det) in dets.iter().enumerate() {
                // Get the relative position of bbox
                let (rx1, ry1, rx2, ry2) = (
                    det.x1 as f64 / iw,
                    det.y1 as f64 / ih,
                    det.x2 as f64 / iw,
                    det.y2 as f64 / ih,
                );
                // Ralative width and height
                let (rw, rh) = (rx2 - rx1, ry2 - ry1);
                // Crop original image by ratio
                let crop = img.crop(
                    (ow * rx1) as u32,
                    (oh * ry1) as u32,
                    (ow * rw) as u32,
                    (oh * rh) as u32,
                );
                // Validate
                if det.cls == 0 && crop.height() > 10 && crop.width() > 10 {
                    // Save the crop image to the specified file path.
                    let _save_res = crop.save(property.path.img.crop.clone());
                    let ocr_dets =
                        self.infer(property.path.img.crop.clone().as_str(), SessionType::Ocr)?;
                    // Collect detected digits
                    let mut digits = vec![];
                    for ocr_det in ocr_dets {
                        digits.push(ocr_det.cls as u8);
                    }
                    new_dets[i].ids = digits;
                }
            }
            Ok(new_dets)
        }
    }

    #[warn(clippy::manual_retain)]
    fn convert_yolo_fmt(
        out: Array<f32, IxDyn>,
    ) -> Result<Vec<super::Detection>, Box<dyn std::error::Error>> {
        // https://github.com/AndreyGermanov/yolov8_onnx_rust
        let mut bboxes = vec![];
        let output = out.slice(s![.., .., 0]);
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();
            if prob < 0.5 {
                continue;
            }
            let cls = class_id as u32;
            let xc = row[0];
            let yc = row[1];
            let w = row[2] as u32;
            let h = row[3] as u32;
            let x1 = (xc - w as f32 / 2.0) as u32;
            let x2 = (xc + w as f32 / 2.0) as u32;
            let y1 = (yc - h as f32 / 2.0) as u32;
            let y2 = (yc + h as f32 / 2.0) as u32;
            let ids: Vec<u8> = vec![];
            bboxes.push(super::Detection {
                x1,
                y1,
                x2,
                y2,
                xc,
                yc,
                cls,
                prob,
                w,
                h,
                ids,
            })
        }
        bboxes.sort_by(|box1, box2| box2.prob.total_cmp(&box1.prob));
        Ok(merge_bboxes(bboxes))
    }

    /// Function to compute the IoU of two rectangles.
    /// https://python-ai-learn.com/2021/02/06/iou/
    ///
    fn iou(r1: Detection, r2: Detection) -> f64 {
        let x1 = r1.x1.max(r2.x1) as f64;
        let y1 = r1.y1.max(r2.y1) as f64;
        let x2 = r1.x2.min(r2.x2) as f64;
        let y2 = r1.y2.min(r2.y2) as f64;
        let w = if x2 - x1 > 0.0 { x2 - x1 } else { 0.0 };
        let h = if y2 - y1 > 0.0 { y2 - y1 } else { 0.0 };
        let intersection = w * h;
        let area_r1 = ((r1.x2 - r1.x1 + 1) * (r1.y2 - r1.y1 + 1)) as f64;
        let area_r2 = ((r2.x2 - r2.x1 + 1) * (r2.y2 - r2.y1 + 1)) as f64;
        let union = area_r1 + area_r2 - intersection;
        intersection / union
    }

    /// Merges bounding boxes whose IoU is greater than or equal to 0.7.
    ///
    fn merge_bboxes(bboxes: Vec<Detection>) -> Vec<Detection> {
        let mut merged_bboxes = Vec::new();
        let mut used = vec![false; bboxes.len()];
        for i in 0..bboxes.len() {
            if used[i] {
                continue;
            }
            let mut merged_bbox = bboxes[i].clone();
            used[i] = true;
            for j in 0..bboxes.len() {
                if used[j] || bboxes[i].cls != bboxes[j].cls {
                    continue;
                }
                if iou(bboxes[i].clone(), bboxes[j].clone()) >= 0.7 {
                    let x1 = merged_bbox.x1.min(bboxes[j].x1);
                    let y1 = merged_bbox.y1.min(bboxes[j].y1);
                    let x2 = merged_bbox.x2.max(bboxes[j].x2);
                    let y2 = merged_bbox.y2.max(bboxes[j].y2);
                    let w = x2 - x1;
                    let h = y2 - y1;
                    let xc = x1 as f32 + w as f32 / 2.0;
                    let yc = y1 as f32 + h as f32 / 2.0;
                    let ids = vec![];

                    merged_bbox = Detection {
                        x1,
                        y1,
                        x2,
                        y2,
                        xc,
                        yc,
                        cls: merged_bbox.cls,
                        prob: merged_bbox.prob,
                        w,
                        h,
                        ids,
                    };
                    used[j] = true;
                }
            }
            merged_bboxes.push(merged_bbox);
        }
        merged_bboxes
    }
}

/// A trait for filtering detection results by class
///
pub trait FilterClass {
    fn filter(dets: &mut [Detection], cls_id: u32) -> Vec<Detection>;
}

/// Roktrack base model's classes
///
#[derive(Debug, Clone, PartialEq)]
pub enum RoktrackClasses {
    PYLON,
    PERSON,
    ROKTRACK,
}
/// Convert int to RoktrackClasses
///
impl RoktrackClasses {
    pub fn from_u32(i: u32) -> Option<RoktrackClasses> {
        match i {
            0 => Some(RoktrackClasses::PYLON),
            1 => Some(RoktrackClasses::PERSON),
            2 => Some(RoktrackClasses::ROKTRACK),
            _ => None,
        }
    }
    pub fn to_u32(&self) -> u32 {
        match self {
            RoktrackClasses::PYLON => 0,
            RoktrackClasses::PERSON => 1,
            RoktrackClasses::ROKTRACK => 2,
        }
    }
}
/// Filter By Class
///
impl FilterClass for RoktrackClasses {
    fn filter(dets: &mut [Detection], cls_id: u32) -> Vec<Detection> {
        let filtered_dets = dets
            .iter()
            .cloned()
            .filter(|det| det.cls == cls_id)
            .collect::<Vec<_>>();
        filtered_dets
    }
}

/// Animal model's classes
///
#[derive(Debug, Clone, PartialEq)]
pub enum AnimalClasses {
    BEAR,
    DEER,
    MONKEY,
    BOAR,
    BADGER,
    CAT,
    CIVET,
    DOG,
    FOX,
    HARE,
    RACOON,
    SQUIRREL,
}
/// AnimalClasses methods
///
impl AnimalClasses {
    pub fn from_u32(i: u32) -> Option<AnimalClasses> {
        match i {
            0 => Some(AnimalClasses::BEAR),
            1 => Some(AnimalClasses::DEER),
            2 => Some(AnimalClasses::MONKEY),
            3 => Some(AnimalClasses::BOAR),
            4 => Some(AnimalClasses::BADGER),
            5 => Some(AnimalClasses::CAT),
            6 => Some(AnimalClasses::CIVET),
            7 => Some(AnimalClasses::DOG),
            8 => Some(AnimalClasses::FOX),
            9 => Some(AnimalClasses::HARE),
            10 => Some(AnimalClasses::RACOON),
            11 => Some(AnimalClasses::SQUIRREL),
            _ => None,
        }
    }
    pub fn to_u32(&self) -> u32 {
        match self {
            AnimalClasses::BEAR => 0,
            AnimalClasses::DEER => 1,
            AnimalClasses::MONKEY => 2,
            AnimalClasses::BOAR => 3,
            AnimalClasses::BADGER => 4,
            AnimalClasses::CAT => 5,
            AnimalClasses::CIVET => 6,
            AnimalClasses::DOG => 7,
            AnimalClasses::FOX => 8,
            AnimalClasses::HARE => 9,
            AnimalClasses::RACOON => 10,
            AnimalClasses::SQUIRREL => 11,
        }
    }
}
/// Filter By Class
///
impl FilterClass for AnimalClasses {
    fn filter(dets: &mut [Detection], cls_id: u32) -> Vec<Detection> {
        dets.iter()
            .cloned()
            .filter(|det| det.cls == cls_id)
            .collect::<Vec<_>>()
    }
}

/// Detection result
///
#[derive(Debug, Clone, PartialEq)]
pub struct Detection {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub xc: f32,
    pub yc: f32,
    pub cls: u32,
    pub prob: f32,
    pub w: u32,
    pub h: u32,
    pub ids: Vec<u8>,
}
/// Detection default method.
///
impl Default for Detection {
    fn default() -> Self {
        Self::new()
    }
}
/// Detection's methods
///
impl Detection {
    // Detection's Constructor

    pub fn new() -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            xc: 0.0,
            yc: 0.0,
            cls: 0,
            prob: 0.0,
            w: 0,
            h: 0,
            ids: vec![],
        }
    }
}

pub mod sort {
    //! Detections sort methods
    //!

    use super::Detection;
    /// sort by right
    ///
    pub fn right(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (-a.xc).partial_cmp(&(-b.xc)).unwrap());
        dets.to_vec()
    }
    /// sort by left
    ///
    pub fn left(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (a.xc).partial_cmp(&(b.xc)).unwrap());
        dets.to_vec()
    }
    /// sort by top
    ///
    pub fn top(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (a.yc).partial_cmp(&(b.yc)).unwrap());
        dets.to_vec()
    }
    /// sort by bottom
    ///
    pub fn bottom(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (-a.yc).partial_cmp(&(-b.yc)).unwrap());
        dets.to_vec()
    }
    /// sort by big
    ///
    pub fn big(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (-(a.h as i32)).partial_cmp(&(-(b.h as i32))).unwrap());
        dets.to_vec()
    }
    /// sort by small
    ///
    pub fn small(dets: &mut [Detection]) -> Vec<Detection> {
        dets.sort_by(|a, b| (a.h).partial_cmp(&(b.h)).unwrap());
        dets.to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sort_detection_test() {
        // center
        let d0 = Detection {
            x1: 155,
            y1: 115,
            x2: 165,
            y2: 125,
            xc: 160.0,
            yc: 120.0,
            cls: 0,
            prob: 0.95,
            w: 10,
            h: 10,
            ids: vec![],
        };
        // left top big
        let d1 = Detection {
            x1: 145,
            y1: 100,
            x2: 155,
            y2: 115,
            xc: 150.0,
            yc: 107.5,
            cls: 0,
            prob: 0.85,
            w: 10,
            h: 15,
            ids: vec![],
        };
        // right bottom small
        let d2 = Detection {
            x1: 165,
            y1: 125,
            x2: 175,
            y2: 130,
            xc: 170.0,
            yc: 127.5,
            cls: 0,
            prob: 0.75,
            w: 10,
            h: 5,
            ids: vec![],
        };
        let mut dets = [d0.clone(), d1.clone(), d2.clone()];
        let right = sort::right(&mut dets).first().unwrap().clone();
        assert_eq!(right, d2.clone());
        let left = sort::left(&mut dets).first().unwrap().clone();
        assert_eq!(left, d1.clone());
        let top = sort::top(&mut dets).first().unwrap().clone();
        assert_eq!(top, d1.clone());
        let bottom = sort::bottom(&mut dets).first().unwrap().clone();
        assert_eq!(bottom, d2.clone());
        let small = sort::small(&mut dets).first().unwrap().clone();
        assert_eq!(small, d2.clone());
        let big: Detection = sort::big(&mut dets).first().unwrap().clone();
        assert_eq!(big, d1.clone());
    }

    #[test]
    fn roktrack_detect_object_test() {
        let detector = onnx::YoloV8::new();
        let dets = detector.infer("asset/img/pylon_10m.jpg", onnx::SessionType::Sz320);
        assert!(dets.unwrap().len() == 2);
        let dets = detector.infer("asset/img/person.jpg", onnx::SessionType::Sz320);
        assert!(dets.unwrap().len() == 1);
    }

    #[test]
    fn animal_detect_object_test() {
        let mut detector = onnx::YoloV8::new();
        detector.sessions = onnx::YoloV8::build_animal_sessions().unwrap();
        let dets = detector.infer("asset/img/bear.jpg", onnx::SessionType::Sz320);
        let dets = AnimalClasses::filter(&mut dets.unwrap(), AnimalClasses::BEAR.to_u32());
        assert!(dets.len() == 1);
        let dets = detector.infer("asset/img/monkey.jpg", onnx::SessionType::Sz320);
        let dets = AnimalClasses::filter(&mut dets.unwrap(), AnimalClasses::MONKEY.to_u32());
        assert!(dets.len() == 2);
    }

    #[test]
    fn pylon_detect_resolution_test() {
        let detector = onnx::YoloV8::new();
        let dets = detector.infer("asset/img/pylon_10m.jpg", onnx::SessionType::Sz320);
        let dets = RoktrackClasses::filter(&mut dets.unwrap(), RoktrackClasses::PYLON.to_u32());
        assert_eq!(dets.len(), 2);
        let dets = detector.infer("asset/img/pylon_10m.jpg", onnx::SessionType::Sz640);
        let dets = RoktrackClasses::filter(&mut dets.unwrap(), RoktrackClasses::PYLON.to_u32());
        assert_eq!(dets.len(), 2);
    }
}
