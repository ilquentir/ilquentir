def make_plot(data, user_id, date_start, date_end):
    '''
    Takes data as pandas DataFrame, user_id as int
    and date_start, date_end as string in format "%Y-%m-%d".

    Returns string with html plot representation.
    '''
    import pandas as pd
    import plotly.graph_objects as go

    from datetime import datetime

    DATE_COL = 'poll_date_about'
    USER_COL = 'user_tg_id'
    ANSW_COL = 'answer_selected_value'
    EVENTS = 'events'

    DATE_PLOT_FORMAT = '%d.%m.%y'

    plot_template = go.layout.Template()
    plot_template.data.bar = [
        go.Bar(
            name='Твоя оценка',
            marker_color='rgb(221, 110, 66)',
            marker_line=dict(width=0.2, color='rgb(255, 255, 179)')
        )
    ]
    plot_template.data.scatter = [
        go.Scatter(
            name='Среднее',
            line = dict(color='rgb(255, 255, 179)', width=8),
            marker=dict(color='rgb(255, 255, 179)', size=5),
            mode='lines+markers'
        ),
    ]

    plot_template.layout = dict(
        font_family="Verdana",
        font_color="rgb(249, 255, 233)",
        font_size=14,

        title_text="Твое настроение",
        title_font_size=24,
        title_font_color="rgb(249, 255, 233)",
        title_x=0.02,
        title_xanchor='auto',

        xaxis_title='',
        yaxis_title='',

        paper_bgcolor='rgb(25, 26, 25)',
        plot_bgcolor='rgb(25, 26, 25)',

        xaxis_showline=True,
        xaxis_linewidth=2,
        xaxis_linecolor='rgb(249, 255, 233)',
        xaxis_showgrid=False,
        xaxis_gridwidth=1,
        xaxis_gridcolor='rgb(249, 255, 233)',
        xaxis_tickangle=45,

        yaxis_showline=True,
        yaxis_linewidth=2,
        yaxis_linecolor='rgb(249, 255, 233)',
        yaxis_showgrid=False,
        yaxis_gridwidth=2,
        yaxis_gridcolor='rgb(249, 255, 233)',

        hoverlabel_bgcolor='rgba(255, 255, 255, 0.5)',
        hoverlabel_font_size=16,
        hoverlabel_font_family="Verdana",
        hoverlabel_font_color='rgba(34, 34, 59, 1)'
    )

    df = data.copy()
    df[DATE_COL] = pd.to_datetime(
        df[DATE_COL],
        # format='%d.%m.%Y %H:%M:%S'
    )
    df[ANSW_COL] = 2 - df[ANSW_COL]
    df = df.drop_duplicates(subset=[USER_COL, DATE_COL])
    df = df[df[DATE_COL].between(date_start, date_end)]

    df_metrics = df.dropna(subset=[ANSW_COL]).groupby(DATE_COL)[ANSW_COL].agg(
        {
            'mean',
        }
    ).round(2)

    df_metrics = df_metrics.join(
        df[df[USER_COL] == user_id].set_index(DATE_COL)[[ANSW_COL, EVENTS]].round(0)
    ).reset_index()

    df_metrics['answ_normalized'] = df_metrics[ANSW_COL] + 3
    df_metrics['mean_normalized'] = df_metrics['mean'] + 3

    fig = go.Figure()

    # create the bar chart
    fig.add_bar(
        x=df_metrics[DATE_COL],
        y=df_metrics['answ_normalized'],
        hovertext=df_metrics[EVENTS],
        hovertemplate='%{y}<br>Что было: %{hovertext}',
        hoverinfo='text',
    )

    # add a line trace for the average
    fig.add_scatter(x=df_metrics[DATE_COL], y=df_metrics['mean_normalized'], mode='lines')

    fig.update_layout(
        yaxis = dict(
            tickvals = [0, 1, 2, 3, 4, 5, 6],
            ticktext = ['', '-2', '-1', '0', '1', '2', '']
        ),
        xaxis = dict(
            tickvals = df_metrics[DATE_COL],
            ticktext = df_metrics[DATE_COL].apply(lambda x: datetime.strftime(x, DATE_PLOT_FORMAT))
        )
    )
    fig.update_layout(template=plot_template)

    fig.update_yaxes(range=[0,5])
    fig.update_layout(bargap=0.0)
    fig.update_layout(hovermode="x unified")
#     #fig.add_hrect(y0=3, y1=5, line_width=0, fillcolor='rgb(119, 150, 109)', opacity=0.3, layer='below')
# #     fig.add_hrect(y0=1, y1=3, line_width=0, fillcolor='rgb(78, 159, 61)', opacity=0.99, layer='below')
# #     fig.add_hrect(y0=0, y1=1, line_width=0, fillcolor='rgb(216, 233, 168)', opacity=0.99, layer='below')

    return fig.to_html(include_plotlyjs='cdn', post_script="document.querySelector('body').style.margin = '0px'; window.dispatchEvent(new Event('resize'));")

if __name__ == '__main__':
    import pandas as pd
    import sys

    wide_file, id_, date_from, date_to = sys.argv[1:]
    data = pd.read_csv(wide_file)

    print(make_plot(data, int(id_), date_from, date_to))
