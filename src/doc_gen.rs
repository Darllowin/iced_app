use std::fs;
use std::path::{Path, PathBuf};
use chrono::NaiveDate;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use umya_spreadsheet::{new_file, writer};
use crate::app::state::{Certificate, Payment, UserInfo};

fn get_reports_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Получаем директорию исполняемого файла
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or("Не удалось получить директорию исполняемого файла")?
        .to_path_buf();

    let reports_dir = exe_dir.join("reports");

    // Создаём папку reports если её нет
    if !reports_dir.exists() {
        fs::create_dir_all(&reports_dir)?;
    }

    Ok(reports_dir)
}

pub fn generate_certificate_html(
    cert: &Certificate,
    student: &UserInfo,
    output_path: &Path,
    assets_dir: &Path,
) -> std::io::Result<()> {
    fn to_file_uri(path: &Path) -> std::io::Result<String> {
        let p = path.canonicalize()?;
        let mut s = p.to_str().unwrap().to_string();

        #[cfg(target_os = "windows")]
        {
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                s = stripped.to_string();
            }
        }

        let s = s.replace("\\", "/");

        #[cfg(target_os = "windows")]
        {
            Ok(format!("file:///{}", s))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(format!("file://{}", s))
        }
    }

    let signature_uri = to_file_uri(&assets_dir.join("images/signature.png"))?;
    let seal_uri = to_file_uri(&assets_dir.join("images/seal.png"))?;

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="UTF-8" />
<title>Сертификат</title>
<style>
  @import url('https://fonts.googleapis.com/css2?family=Fira+Sans&display=swap');
  @page {{
    size: A4;
    margin: 0;
  }}
  body {{
    font-family: 'Fira Sans', sans-serif;
    background: #282828;
    margin: 0;
    padding: 0;
    color: #ebdbb2;
    width: 794px;
    height: 1123px;
  }}
  .certificate {{
    border: 8px solid #458588;
    padding: 40px;
    width: 718px;
    height: 1043px;
    margin: 0 auto;
    background: #3c3836;
    box-shadow: 0 0 15px rgba(0,0,0,0.5);
    text-align: center;
    border-radius: 12px;
    position: relative;
    box-sizing: border-box;
  }}
  h1 {{
    font-size: 56px;
    color: #fabd2f;
    margin-bottom: 20px;
  }}
  p {{
    font-size: 20px;
    margin: 15px 0;
  }}
  .student-name {{
    font-weight: bold;
    font-size: 40px;
    color: #83a598;
    margin: 30px 0;
  }}
  .course-title {{
    font-weight: bold;
    font-size: 34px;
    margin: 20px 0;
    color: #b8bb26;
  }}
  .grade {{
    font-size: 22px;
    margin: 15px 0;
    font-weight: 600;
    color: #fb4934;
  }}
  .footer {{
    margin-top: 60px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 30px;
  }}
  .signature-box {{
    width: 40%;
    border-top: 1px solid #928374;
    padding-top: 10px;
    font-size: 18px;
    font-weight: 600;
    color: #ebdbb2;
    position: relative;
  }}
  .signature-img {{
    position: absolute;
    top: -70px;
    left: 0;
    width: 150px;
    height: auto;
  }}
  .stamp-img {{
    width: 120px;
    height: auto;
    opacity: 0.5;
    filter: drop-shadow(0 0 2px rgba(0,0,0,0.3));
    margin-right: 10px;
  }}
  .date {{
    font-size: 16px;
    color: #a89984;
    margin-top: 40px;
  }}
  .decorative-line {{
    width: 60px;
    height: 4px;
    background: #d79921;
    margin: 20px auto;
    border-radius: 2px;
  }}
</style>
</head>
<body>
  <div class="certificate">
    <h1>СЕРТИФИКАТ</h1>
    <div class="decorative-line"></div>
    <p>Настоящим подтверждается, что</p>
    <p class="student-name">{student_name}</p>
    <p>успешно завершил(а) курс</p>
    <p class="course-title">{course_title}</p>
    <p class="grade">С оценкой: {grade}</p>
    <div class="decorative-line"></div>

    <div class="footer">
      <div class="signature-box">
        <img src="{signature_uri}" alt="Подпись директора" class="signature-img" />
        Подпись директора
      </div>
      <img src="{seal_uri}" alt="Печать" class="stamp-img" />
    </div>

    <p class="date">Дата выдачи: {issue_date}</p>
  </div>
</body>
</html>"#,
        student_name = student.name,
        course_title = cert.course_title,
        grade = cert.grade,
        issue_date = cert.issue_date,
        signature_uri = signature_uri,
        seal_uri = seal_uri,
    );

    fs::write(output_path, html_content)
}

pub fn generate_pdf_from_html(
    html_path: &Path,
    output_pdf: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true)
            .build()
            .unwrap(),
    )?;

    let tab = browser.new_tab()?;

    // Получаем абсолютный путь
    let abs_path = fs::canonicalize(html_path)?;
    let mut abs_str = abs_path
        .to_str()
        .ok_or("Путь содержит недопустимые символы (не UTF-8)")?
        .to_string();

    #[cfg(target_os = "windows")]
    {
        // Убираем префикс \\?\ если он есть
        if let Some(stripped) = abs_str.strip_prefix(r"\\?\") {
            abs_str = stripped.to_string();
        }
        // Заменяем \ на /
        abs_str = abs_str.replace("\\", "/");
    }

    // Формируем корректный file:// URL
    #[cfg(target_os = "windows")]
    let url = format!("file:///{}", abs_str);

    #[cfg(not(target_os = "windows"))]
    let url = format!("file://{}", abs_str);

    println!("DEBUG: Навигация headless_chrome к URL: {}", url);

    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;

    let pdf_data = tab.print_to_pdf(Default::default())?;
    fs::write(output_pdf, &pdf_data)?;

    Ok(())
}
pub fn generate_payment_report_html(
    payments: &[Payment],
    from: &str,
    to: &str,
    output_path: &Path,
) -> std::io::Result<()> {
    use std::collections::HashMap;
    use std::fs;

    let mut type_sums: HashMap<String, f64> = HashMap::new();
    for p in payments {
        *type_sums.entry(p.payment_type.clone()).or_insert(0.0) += p.amount;
    }

    // Преобразуем from и to в формат "дд:мм:гггг"
    let from_formatted = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map(|d| d.format("%d:%m:%Y").to_string())
        .unwrap_or_else(|_| from.to_string());

    let to_formatted = NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map(|d| d.format("%d:%m:%Y").to_string())
        .unwrap_or_else(|_| to.to_string());

    let mut table_rows = String::new();
    for p in payments {
        let formatted_date = NaiveDate::parse_from_str(&p.date, "%Y-%m-%d")
            .map(|d| d.format("%d:%m:%Y").to_string())
            .unwrap_or_else(|_| p.date.clone());

        table_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2} ₽</td></tr>",
            formatted_date, p.student_name, p.course_title, p.payment_type, p.amount
        ));
    }

    let chart_labels: Vec<_> = type_sums.keys().cloned().collect();
    let chart_data: Vec<_> = type_sums.values().map(|v| *v).collect();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="UTF-8">
<title>Отчёт по платежам</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<style>
  /* Gruvbox цвета */
  :root {{
    --bright_red: #fb4934;
    --bright_green: #b8bb26;
    --bright_yellow: #fabd2f;
    --bright_blue: #83a598;
    --bright_purple: #d3869b;
    --bright_aqua: #8ec07c;
    --bright_orange: #fe8019;
  }}

  /* Светлая тема Gruvbox Light */
  body.light {{
    --bg: #fbf1c7;
    --bg-alt: #f2e5bc;
    --fg: #3c3836;
    --fg-muted: #7c6f64;
    --table-border: #d5c4a1;
    --table-header-bg: #ebdbb2;
    --hover-bg: #fabd2f;
  }}

  /* Тёмная тема Gruvbox Dark */
  body.dark {{
    --bg: #282828;
    --bg-alt: #3c3836;
    --fg: #ebdbb2;
    --fg-muted: #a89984;
    --table-border: #504945;
    --table-header-bg: #504945;
    --hover-bg: #fabd2f;
  }}

  body {{
    font-family: Arial, sans-serif;
    background-color: var(--bg);
    color: var(--fg);
    padding: 30px;
    margin: 0;
    transition: background-color 0.3s, color 0.3s;
  }}

  h1 {{
    text-align: center;
    color: var(--bright_yellow);
    transition: color 0.3s;
  }}

  p {{
    font-size: 1.1em;
    text-align: center;
    margin-bottom: 20px;
    color: var(--fg-muted);
    transition: color 0.3s;
  }}

  table {{
    width: 100%;
    border-collapse: collapse;
    background-color: var(--bg-alt);
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 0 8px rgba(0,0,0,0.15);
    transition: background-color 0.3s;
  }}

  th, td {{
    border: 1px solid var(--table-border);
    padding: 8px;
    text-align: left;
    color: var(--fg);
    transition: color 0.3s, border-color 0.3s;
  }}

  th {{
    background-color: var(--table-header-bg);
    color: var(--bright_yellow);
    font-weight: 600;
    letter-spacing: 0.03em;
    transition: background-color 0.3s, color 0.3s;
  }}

  tbody tr:nth-child(even) {{
    background-color: var(--bg);
  }}

  tbody tr:hover {{
    background-color: var(--hover-bg);
    color: var(--bg);
    cursor: default;
    transition: background-color 0.3s, color 0.3s;
  }}

  .chart-container {{
    width: 500px;
    max-width: 90vw;
    margin: 40px auto 0;
    background-color: var(--bg-alt);
    padding: 15px;
    border-radius: 10px;
    box-shadow: 0 0 12px rgba(0,0,0,0.1);
    transition: background-color 0.3s;
  }}

  /* Переключатель темы */
  .theme-switch-wrapper {{
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
  }}

  .theme-switch {{
    display: inline-block;
    height: 30px;
    position: relative;
    width: 60px;
  }}

  .theme-switch input {{
    display:none;
  }}

  .slider {{
    background-color: #ccc;
    bottom: 0;
    cursor: pointer;
    left: 0;
    position: absolute;
    right: 0;
    top: 0;
    transition: 0.4s;
    border-radius: 30px;
  }}

  .slider:before {{
    background-color: white;
    bottom: 4px;
    content: "";
    height: 22px;
    left: 4px;
    position: absolute;
    transition: 0.4s;
    width: 22px;
    border-radius: 50%;
  }}

  input:checked + .slider {{
    background-color: var(--bright_yellow);
  }}

  input:checked + .slider:before {{
    transform: translateX(30px);
  }}
</style>
</head>
<body>
  <div class="theme-switch-wrapper" title="Переключить тему Gruvbox Light/Dark">
    <label class="theme-switch" for="checkbox">
      <input type="checkbox" id="checkbox" />
      <span class="slider"></span>
    </label>
  </div>

  <h1>Отчёт по платежам</h1>
  <p><strong>Период:</strong> {from_formatted} — {to_formatted}</p>

  <table>
    <thead>
      <tr><th>Дата</th><th>Студент</th><th>Курс</th><th>Тип</th><th>Сумма</th></tr>
    </thead>
    <tbody>
      {table_rows}
    </tbody>
  </table>

  <div class="chart-container">
    <canvas id="paymentChart"></canvas>
  </div>

<script>
  const toggleSwitch = document.getElementById('checkbox');
  const currentTheme = localStorage.getItem('theme') || 'dark';
  document.body.classList.add(currentTheme);
  toggleSwitch.checked = currentTheme === 'light';

  toggleSwitch.addEventListener('change', function() {{
    if (this.checked) {{
      document.body.classList.replace('dark', 'light');
      localStorage.setItem('theme', 'light');
    }} else {{
      document.body.classList.replace('light', 'dark');
      localStorage.setItem('theme', 'dark');
    }}
    updateChartColors();
  }});

  const ctx = document.getElementById('paymentChart').getContext('2d');

  const colorsLight = ['#fabd2f', '#83a598', '#fb4934', '#b8bb26', '#d3869b'];
  const colorsDark = ['#fabd2f', '#83a598', '#fb4934', '#b8bb26', '#d3869b'];

  function getColors() {{
    return document.body.classList.contains('light') ? colorsLight : colorsDark;
  }}

  let chart = new Chart(ctx, {{
    type: 'pie',
    data: {{
      labels: {labels:?},
      datasets: [{{
        label: 'Суммы по типам',
        data: {data:?},
        backgroundColor: getColors(),
        borderColor: document.body.classList.contains('light') ? '#fbf1c7' : '#282828',
        borderWidth: 2
      }}]
    }},
    options: {{
      responsive: true,
      plugins: {{
        legend: {{
          position: 'bottom',
          labels: {{
            color: getComputedStyle(document.body).getPropertyValue('--fg').trim()
          }}
        }},
      }},
    }}
  }});

  function updateChartColors() {{
    chart.data.datasets[0].backgroundColor = getColors();
    chart.data.datasets[0].borderColor = document.body.classList.contains('light') ? '#fbf1c7' : '#282828';
    chart.options.plugins.legend.labels.color = getComputedStyle(document.body).getPropertyValue('--fg').trim();
    chart.update();
  }}
</script>

</body>
</html>
"#,
        from_formatted = from_formatted,
        to_formatted = to_formatted,
        table_rows = table_rows,
        labels = chart_labels,
        data = chart_data,
    );

    fs::write(output_path, html)
}

pub fn generate_payment_report(
    payments: &[Payment],
    from: &str,
    to: &str,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::NaiveDate;

    let from_date = NaiveDate::parse_from_str(from, "%Y-%m-%d")?;
    let to_date = NaiveDate::parse_from_str(to, "%Y-%m-%d")?;

    let filtered: Vec<_> = payments.iter()
        .filter(|p| {
            NaiveDate::parse_from_str(&p.date, "%Y-%m-%d")
                .map(|d| d >= from_date && d <= to_date)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    if filtered.is_empty() {
        println!("Нет платежей за указанный период");
        return Ok(());
    }

    let html_path = output_dir.join("payment_report.html");
    let pdf_path = output_dir.join("payment_report.pdf");

    generate_payment_report_html(&filtered, from, to, &html_path)?;
    generate_pdf_from_html(&html_path, &pdf_path)?;

    println!("PDF-отчёт сгенерирован: {:?}", pdf_path);
    Ok(())
}
fn col_to_letter(col: usize) -> String {
    // Простая конвертация индекса столбца (1-based) в букву, подходит для A-Z
    // Для >26 столбцов нужно расширить
    ((b'A' + (col as u8) - 1) as char).to_string()
}

pub fn generate_payment_excel_report(
    payments: &[Payment],
    from: &NaiveDate,
    to: &NaiveDate,
    _output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {

    let reports_dir = get_reports_dir()?;

    // Формируем имя файла с датами, например "payment_report_2025-05-01_2025-05-26.xlsx"
    let filename = format!("payment_report_{}_{}.xlsx", from, to);
    let output_path = reports_dir.join(filename);
    let mut book = new_file();
    let sheet = book.get_sheet_by_name_mut("Sheet1").unwrap();

    // Заголовок
    sheet.get_cell_mut("A1").set_value("Отчёт по платежам");
    sheet.get_cell_mut("A2")
        .set_value(format!("Период: {} — {}", from, to));

    // Заголовки таблицы
    let headers = ["Дата", "Студент", "Курс", "Тип", "Сумма"];
    for (i, header) in headers.iter().enumerate() {
        let col_letter = col_to_letter(i + 1);
        let cell_address = format!("{}4", col_letter);
        sheet.get_cell_mut(&*cell_address).set_value(*header);
    }

    // Данные
    for (i, payment) in payments.iter().enumerate() {
        let row = i + 5;

        let date = NaiveDate::parse_from_str(&payment.date, "%Y-%m-%d")
            .map(|d| d.format("%d.%m.%Y").to_string())
            .unwrap_or_else(|_| payment.date.clone());

        let values = [
            date,
            payment.student_name.clone(),
            payment.course_title.clone(),
            payment.payment_type.clone(),
            payment.amount.to_string(),
        ];

        for (j, value) in values.iter().enumerate() {
            let col_letter = col_to_letter(j + 1);
            let cell_address = format!("{}{}", col_letter, row);
            sheet.get_cell_mut(&*cell_address).set_value(value);
        }
    }

    writer::xlsx::write(&book, output_path)?;

    Ok(())
}
pub fn generate_certificate_report_html(
    certificates: &[Certificate],
    from: &str,
    to: &str,
    output_path: &Path,
) -> std::io::Result<()> {
    use std::collections::HashMap;
    use std::fs;

    // Подсчёт количества сертификатов по оценкам (для графика)
    let mut grade_counts: HashMap<String, usize> = HashMap::new();
    for cert in certificates {
        *grade_counts.entry(cert.grade.clone()).or_insert(0) += 1;
    }

    // Форматируем даты для отображения
    let from_formatted = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| from.to_string());

    let to_formatted = NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| to.to_string());

    // Формируем строки таблицы
    let mut table_rows = String::new();
    for cert in certificates {
        let formatted_date = NaiveDate::parse_from_str(&cert.issue_date, "%Y-%m-%d")
            .map(|d| d.format("%d.%m.%Y").to_string())
            .unwrap_or_else(|_| cert.issue_date.clone());

        table_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            cert.id, cert.student_name, cert.course_title, formatted_date, cert.grade
        ));
    }

    // Для графика: метки и данные
    let chart_labels: Vec<_> = grade_counts.keys().cloned().collect();
    let chart_data: Vec<_> = grade_counts.values().map(|v| *v as f64).collect();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="UTF-8">
<title>Отчёт по сертификатам</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<style>
  /* Gruvbox цвета (согласно https://github.com/morhetz/gruvbox) */

  /* Цвета, используемые в обеих темах */
  :root {{
    --bright_red: #fb4934;
    --bright_green: #b8bb26;
    --bright_yellow: #fabd2f;
    --bright_blue: #83a598;
    --bright_purple: #d3869b;
    --bright_aqua: #8ec07c;
    --bright_orange: #fe8019;
  }}

  /* Светлая тема Gruvbox Light */
  body.light {{
    --bg: #fbf1c7;
    --bg-alt: #f2e5bc;
    --fg: #3c3836;
    --fg-muted: #7c6f64;
    --table-border: #d5c4a1;
    --table-header-bg: #ebdbb2;
    --hover-bg: #fabd2f;
  }}

  /* Тёмная тема Gruvbox Dark */
  body.dark {{
    --bg: #282828;
    --bg-alt: #3c3836;
    --fg: #ebdbb2;
    --fg-muted: #a89984;
    --table-border: #504945;
    --table-header-bg: #504945;
    --hover-bg: #fabd2f;
  }}

  body {{
    font-family: "Fira Sans", Arial, sans-serif;
    background-color: var(--bg);
    color: var(--fg);
    padding: 20px;
    margin: 0;
    transition: background-color 0.3s, color 0.3s;
  }}

  h1 {{
    text-align: center;
    margin-bottom: 5px;
    color: var(--bright_yellow);
    font-weight: 700;
    transition: color 0.3s;
  }}

  p {{
    text-align: center;
    margin-top: 0;
    margin-bottom: 20px;
    font-size: 1.1em;
    color: var(--fg-muted);
    transition: color 0.3s;
  }}

  table {{
    width: 100%;
    border-collapse: collapse;
    background-color: var(--bg-alt);
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 0 10px rgba(0,0,0,0.15);
    transition: background-color 0.3s;
  }}

  th, td {{
    border: 1px solid var(--table-border);
    padding: 10px 12px;
    text-align: left;
    color: var(--fg);
    transition: color 0.3s, border-color 0.3s;
  }}

  th {{
    background-color: var(--table-header-bg);
    font-weight: 600;
    letter-spacing: 0.03em;
    transition: background-color 0.3s;
  }}

  tr:nth-child(even) {{
    background-color: var(--bg);
  }}

  tr:hover {{
    background-color: var(--hover-bg);
    color: var(--bg);
    cursor: default;
    transition: background-color 0.3s, color 0.3s;
  }}

  .chart-container {{
    width: 400px;
    max-width: 90vw;
    margin: 30px auto 0;
    background-color: var(--bg-alt);
    padding: 15px;
    border-radius: 10px;
    box-shadow: 0 0 12px rgba(0,0,0,0.1);
    transition: background-color 0.3s;
  }}

  /* Переключатель темы */
  .theme-switch-wrapper {{
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
  }}

  .theme-switch {{
    display: inline-block;
    height: 30px;
    position: relative;
    width: 60px;
  }}

  .theme-switch input {{
    display:none;
  }}

  .slider {{
    background-color: #ccc;
    bottom: 0;
    cursor: pointer;
    left: 0;
    position: absolute;
    right: 0;
    top: 0;
    transition: 0.4s;
    border-radius: 30px;
  }}

  .slider:before {{
    background-color: white;
    bottom: 4px;
    content: "";
    height: 22px;
    left: 4px;
    position: absolute;
    transition: 0.4s;
    width: 22px;
    border-radius: 50%;
  }}

  input:checked + .slider {{
    background-color: var(--bright_yellow);
  }}

  input:checked + .slider:before {{
    transform: translateX(30px);
  }}

</style>
</head>
<body>
  <div class="theme-switch-wrapper" title="Переключить тему Gruvbox Light/Dark">
    <label class="theme-switch" for="checkbox">
      <input type="checkbox" id="checkbox" />
      <span class="slider"></span>
    </label>
  </div>

  <h1>Отчёт по сертификатам</h1>
  <p><strong>Период:</strong> {from_formatted} — {to_formatted}</p>

  <table>
    <thead>
      <tr><th>ID</th><th>Студент</th><th>Курс</th><th>Дата выдачи</th><th>Оценка</th></tr>
    </thead>
    <tbody>
      {table_rows}
    </tbody>
  </table>

  <div class="chart-container">
    <canvas id="gradeChart"></canvas>
  </div>

<script>
  // Сохранение и восстановление темы
  const toggleSwitch = document.getElementById('checkbox');
  const currentTheme = localStorage.getItem('theme') || 'dark';
  document.body.classList.add(currentTheme);
  toggleSwitch.checked = currentTheme === 'light';

  toggleSwitch.addEventListener('change', function() {{
    if (this.checked) {{
      document.body.classList.replace('dark', 'light');
      localStorage.setItem('theme', 'light');
    }} else {{
      document.body.classList.replace('light', 'dark');
      localStorage.setItem('theme', 'dark');
    }}
  }});

  // Chart.js
  const ctx = document.getElementById('gradeChart').getContext('2d');

  // Цвета для графика, подстроятся под тему (используем Gruvbox яркие цвета)
  const colorsLight = ['#fabd2f','#fe8019','#fb4934','#b8bb26','#8ec07c','#83a598'];
  const colorsDark = ['#fabd2f','#fe8019','#fb4934','#b8bb26','#8ec07c','#83a598'];

  const getColors = () => document.body.classList.contains('light') ? colorsLight : colorsDark;

  let chart = new Chart(ctx, {{
    type: 'pie',
    data: {{
      labels: {labels:?},
      datasets: [{{
        label: 'Количество сертификатов по оценкам',
        data: {data:?},
        backgroundColor: getColors(),
        borderColor: document.body.classList.contains('light') ? '#fbf1c7' : '#282828',
        borderWidth: 2
      }}]
    }},
    options: {{
      responsive: true,
      plugins: {{
        legend: {{
          position: 'bottom',
          labels: {{
            color: getComputedStyle(document.body).getPropertyValue('--fg').trim()
          }}
        }},
      }},
    }}
  }});

  // Обновление цветов графика при смене темы
  toggleSwitch.addEventListener('change', () => {{
    chart.data.datasets[0].backgroundColor = getColors();
    chart.data.datasets[0].borderColor = document.body.classList.contains('light') ? '#fbf1c7' : '#282828';
    chart.options.plugins.legend.labels.color = getComputedStyle(document.body).getPropertyValue('--fg').trim();
    chart.update();
  }});
</script>
</body>
</html>
"#,
        from_formatted = from_formatted,
        to_formatted = to_formatted,
        table_rows = table_rows,
        labels = chart_labels,
        data = chart_data
    );

    fs::write(output_path, html)
}


pub fn generate_certificate_report(
    certificates: &[Certificate],
    from: &str,
    to: &str,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if certificates.is_empty() {
        println!("Нет сертификатов за указанный период");
        return Ok(());
    }

    let html_path = output_dir.join("certificate_report.html");
    let pdf_path = output_dir.join("certificate_report.pdf");

    generate_certificate_report_html(certificates, from, to, &html_path)?;
    generate_pdf_from_html(&html_path, &pdf_path)?;

    println!("PDF-отчёт по сертификатам сгенерирован: {:?}", pdf_path);
    Ok(())
}
pub fn generate_certificate_excel_report(
    certificates: &[Certificate],
    from: &NaiveDate,
    to: &NaiveDate,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use umya_spreadsheet::{new_file, writer};

    // Создаем директорию, если нет
    fs::create_dir_all(output_dir)?;
    


    let filtered: Vec<_> = certificates.iter()
        .filter(|c| {
            NaiveDate::parse_from_str(&c.issue_date, "%Y-%m-%d")
                .map(|d| d >= *from && d <= *to)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    if filtered.is_empty() {
        println!("Нет сертификатов для Excel отчёта за указанный период");
        return Ok(());
    }

    let filename = format!("certificate_report_{}_{}.xlsx", from, to);
    let output_path = output_dir.join(filename);
    let mut book = new_file();
    let sheet = book.get_sheet_by_name_mut("Sheet1").unwrap();

    // Заголовок
    sheet.get_cell_mut("A1").set_value("Отчёт по сертификатам");
    sheet.get_cell_mut("A2")
        .set_value(format!("Период: {} — {}", from, to));

    // Заголовки таблицы
    let headers = ["Студент", "Курс", "Дата выдачи", "Оценка"];
    for (i, header) in headers.iter().enumerate() {
        let col_letter = col_to_letter(i + 1);
        let cell_address = format!("{}4", col_letter);
        sheet.get_cell_mut(&*cell_address).set_value(*header);
    }

    // Данные
    for (i, cert) in filtered.iter().enumerate() {
        let row = i + 5;

        let date = NaiveDate::parse_from_str(&cert.issue_date, "%Y-%m-%d")
            .map(|d| d.format("%d.%m.%Y").to_string())
            .unwrap_or_else(|_| cert.issue_date.clone());

        let values = [
            cert.student_name.clone(),
            cert.course_title.clone(),
            date,
            cert.grade.clone(),
        ];

        for (j, value) in values.iter().enumerate() {
            let col_letter = col_to_letter(j + 1);
            let cell_address = format!("{}{}", col_letter, row);
            sheet.get_cell_mut(&*cell_address).set_value(value);
        }
    }

    writer::xlsx::write(&book, output_path)?;

    println!("Excel отчёт по сертификатам сгенерирован");

    Ok(())
}

