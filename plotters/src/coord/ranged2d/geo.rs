use crate::coord::cartesian::MeshLine;
use crate::coord::ranged1d::{AsRangedCoord, KeyPointHint};
use crate::coord::{cartesian::Cartesian2d, types::RangedCoordf64};
use crate::prelude::{
    ChartBuilder, ChartContext, CoordTranslate, DrawingArea, DrawingAreaErrorKind, DrawingBackend,
};
use proj::{Proj, ProjError};
use std::ops::Range as SRange;

use thiserror::Error;

type Range = (f64, f64);

#[derive(Error, Debug)]
pub enum CoordError {
    #[error("Un")]
    Uninital,
    #[error("")]
    ProjError {
        #[from]
        source: ProjError,
    },
}

#[derive(Clone)]
pub struct LatLonCoord<T>
where
    T: ProjectionS,
{
    pub lon: Option<Range>,
    pub lat: Option<Range>,
    x: Range,
    y: Range,
    cartesian: Cartesian2d<RangedCoordf64, RangedCoordf64>,
    proj: T,
}

impl<T: ProjectionS> LatLonCoord<T> {
    pub fn new(
        lon: Option<Range>,
        lat: Option<Range>,
        actual: (SRange<i32>, SRange<i32>),
        proj: T,
    ) -> Self {
        let _box = proj.bbox(lon, lat).unwrap();
        Self {
            lon: lon,
            lat: lat,
            x: _box.0,
            y: _box.1,
            cartesian: Cartesian2d::new(_box.0 .0.._box.0 .1, _box.1 .0.._box.1 .1, actual),
            proj: proj,
        }
    }
}

impl<T: ProjectionS> CoordTranslate for LatLonCoord<T> {
    type From = Range;
    fn translate(&self, from: &Self::From) -> plotters_backend::BackendCoord {
        self.cartesian.translate(&self.proj.map(*from))
    }
}

#[derive(Clone, Debug)]
pub enum Projection {
    PlateCarree,
    LambertConformal,
    LambertCylindrical,
    Mercator,
}

pub trait ProjectionS {
    fn bbox(
        &self,
        x_ranged: Option<(f64, f64)>,
        y_ranged: Option<(f64, f64)>,
    ) -> Result<(Range, Range), CoordError>;

    fn map(&self, v: Range) -> Range;
}

pub struct Mercator {
    central_lon: f64,
    min_latitude: f64,
    max_latitude: f64,

    false_easting: f64,
    false_northing: f64,
    latitude_true_scale: f64,

    proj_marker: Option<Proj>,
}

fn proj_string<'a>(vs: Vec<(&'a str, &'a str)>) -> String {
    vs.into_iter()
        .map(|(option, value)| format!("+{}={}", option, value))
        .collect::<Vec<String>>()
        .join(" ")
}

impl Mercator {
    pub fn new() -> Self {
        Self {
            central_lon: 0.0,
            min_latitude: -80.0,
            max_latitude: 84.0,
            false_easting: 0.0,
            false_northing: 0.0,
            latitude_true_scale: 0.0,
            proj_marker: None,
        }
    }

    pub fn build(mut self) -> Self {
        let _central_lon = &self.central_lon.to_string();
        let _false_easting = &self.false_easting.to_string();
        let _false_northing = &self.false_northing.to_string();

        let input = vec![
            ("proj", "merc"),
            ("lon_0", _central_lon.as_str()),
            ("x_0", _false_easting.as_str()),
            ("y_0", _false_northing.as_str()),
            ("units", "m"),
        ];
        let _proj_string = proj_string(input);

        self.proj_marker = Some(Proj::new(_proj_string.as_str()).unwrap());

        self
    }
}

impl ProjectionS for Mercator {
    fn bbox(
        &self,
        x_ranged: Option<(f64, f64)>,
        y_ranged: Option<(f64, f64)>,
    ) -> Result<(Range, Range), CoordError> {
        let _proj_transformer = self.proj_marker.as_ref().ok_or(CoordError::Uninital)?;
        let (x_min, x_max) = x_ranged.map_or((-180.0, 180.0), |v| v);
        let (y_min, y_max) = y_ranged.map_or((self.min_latitude, self.max_latitude), |v| v);

        let bl = _proj_transformer.convert((x_min, y_min))?;

        let rt = _proj_transformer.convert((x_max, y_max))?;

        Ok(((bl.0, rt.0), (bl.1, rt.1)))
    }

    fn map(&self, v: Range) -> Range {
        let _proj_transformer = self.proj_marker.as_ref().unwrap();
        _proj_transformer.convert(v).unwrap()
    }
}

pub trait GeoCoordTrait<'a, DB: DrawingBackend> {
    fn build_geo_coord<X: AsRangedCoord, Y: AsRangedCoord>(
        &mut self,
        x_spec: X,
        y_spec: Y,
    ) -> Result<
        ChartContext<'a, DB, Cartesian2d<X::CoordDescType, Y::CoordDescType>>,
        DrawingAreaErrorKind<DB::ErrorType>,
    >;
}
